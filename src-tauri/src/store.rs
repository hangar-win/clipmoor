use std::path::{Path, PathBuf};
use std::sync::Arc;

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::classify::Kind;
use crate::clip::ClipFormats;
use crate::crypto::{self, CryptoError, ENC_V1, ItemKey, Keys};

#[derive(Debug)]
pub enum StoreError {
    Sql(rusqlite::Error),
    Crypto(CryptoError),
    Json(serde_json::Error),
    Locked,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Sql(e) => write!(f, "sqlite: {e}"),
            StoreError::Crypto(e) => write!(f, "crypto: {e}"),
            StoreError::Json(e) => write!(f, "meta: {:?}", e.classify()),
            StoreError::Locked => f.write_str("item is locked"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        StoreError::Sql(e)
    }
}

impl From<CryptoError> for StoreError {
    fn from(e: CryptoError) -> Self {
        StoreError::Crypto(e)
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(e: serde_json::Error) -> Self {
        StoreError::Json(e)
    }
}

pub type Result<T> = std::result::Result<T, StoreError>;

type ExistingRow = (i64, Vec<u8>, Vec<u8>, [bool; 3]);
type ItemRow = (i64, i64, bool, bool, Vec<u8>, Vec<u8>, Option<Vec<u8>>);

const SCHEMA_V1: &str = "BEGIN;
CREATE TABLE items (
  id INTEGER PRIMARY KEY,
  last_used INTEGER NOT NULL,
  pinned INTEGER NOT NULL DEFAULT 0,
  locked INTEGER NOT NULL DEFAULT 0,
  dedup BLOB UNIQUE,
  enc_v INTEGER NOT NULL,
  salt BLOB NOT NULL,
  meta BLOB NOT NULL,
  plain BLOB, html BLOB, rtf BLOB, png BLOB, thumb BLOB
);
CREATE TABLE groups (
  id INTEGER PRIMARY KEY,
  locked INTEGER NOT NULL DEFAULT 0,
  enc_v INTEGER NOT NULL,
  salt BLOB NOT NULL,
  name BLOB NOT NULL
);
PRAGMA user_version = 1;
COMMIT;";

const SCHEMA_V2: &str = "BEGIN;
CREATE TABLE groups_v2 (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  locked INTEGER NOT NULL DEFAULT 0,
  enc_v INTEGER NOT NULL,
  salt BLOB NOT NULL,
  name BLOB NOT NULL
);
INSERT INTO groups_v2 (id, locked, enc_v, salt, name) SELECT id, locked, enc_v, salt, name FROM groups;
DROP TABLE groups;
ALTER TABLE groups_v2 RENAME TO groups;
PRAGMA user_version = 2;
COMMIT;";

const PREVIEW_CHARS: usize = 300;
const SEARCH_BYTES: usize = 64 * 1024;
const DAY: i64 = 86_400;

pub struct NewClip {
    pub formats: ClipFormats,
    pub kind: Kind,
    pub source: String,
    pub thumb: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ItemMeta {
    pub kind: Kind,
    pub source: String,
    pub size: u64,
    pub preview: String,
    pub group: Option<i64>,
    pub has_image: bool,
    #[serde(default)]
    pub own_lock: bool,
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LockSource {
    All,
    Group,
}

pub struct Item {
    pub id: i64,
    pub last_used: i64,
    pub pinned: bool,
    pub locked: bool,
    pub meta: Option<ItemMeta>,
    pub search: Option<Arc<str>>,
    pub locked_by: Option<LockSource>,
    pub text_tag: Option<Vec<u8>>,
}

#[derive(Default)]
pub struct Loaded {
    pub items: Vec<Item>,
    pub unreadable: usize,
    pub locked_ok: usize,
    pub locked_bad: usize,
}

pub struct Group {
    pub id: i64,
    pub name: String,
    pub locked: bool,
}

pub struct Store {
    conn: Connection,
    pub lock_all: bool,
    vacuum_stamp: Option<PathBuf>,
}

fn search_text(plain: &str) -> Arc<str> {
    let mut end = plain.len().min(SEARCH_BYTES);
    while !plain.is_char_boundary(end) {
        end -= 1;
    }
    crate::search::fold(&plain[..end]).into()
}

fn row_key(k: &Keys, locked: bool, salt: &[u8]) -> Result<ItemKey> {
    let salt: &[u8; 32] = salt.try_into().map_err(|_| CryptoError::Malformed)?;
    let root: &[u8] = if locked { k.lock_root.as_ref().ok_or(StoreError::Locked)?.as_bytes() } else { k.master.as_bytes() };
    Ok(crypto::item_key(root, salt))
}

fn open_blob(key: &ItemKey, b: Option<Vec<u8>>) -> Result<Option<Vec<u8>>> {
    Ok(b.map(|b| crypto::open(key, &b)).transpose()?)
}

fn open_text(key: &ItemKey, b: Option<Vec<u8>>) -> Result<Option<String>> {
    Ok(open_blob(key, b)?.map(|b| String::from_utf8(b).map_err(|_| CryptoError::Malformed)).transpose()?)
}

fn open_meta(key: &ItemKey, b: &[u8]) -> Result<ItemMeta> {
    Ok(serde_json::from_slice(&crypto::open(key, b)?)?)
}

fn dedup_tag(k: &Keys, locked: bool, plain: Option<&[u8]>, png: Option<&[u8]>) -> Result<Option<Vec<u8>>> {
    let Some(data) = plain.or(png) else { return Ok(None) };
    let tag = if locked { crypto::dedup_tag(&crypto::lock_dedup_key(k.lock_root.as_ref().ok_or(StoreError::Locked)?), data) } else { crypto::dedup_tag(&k.dedup, data) };
    Ok(Some(tag.to_vec()))
}

fn group_locked(conn: &Connection, group: Option<i64>) -> Result<bool> {
    let Some(g) = group else { return Ok(false) };
    Ok(conn.query_row("SELECT locked FROM groups WHERE id = ?1", [g], |r| r.get(0)).optional()?.unwrap_or(false))
}

fn read_meta(conn: &Connection, k: &Keys, id: i64) -> Result<(bool, ItemKey, ItemMeta)> {
    let (locked, salt, meta): (bool, Vec<u8>, Vec<u8>) = conn.query_row("SELECT locked, salt, meta FROM items WHERE id = ?1", [id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
    let key = row_key(k, locked, &salt)?;
    let meta = open_meta(&key, &meta)?;
    Ok((locked, key, meta))
}

fn write_meta(conn: &Connection, key: &ItemKey, id: i64, m: &ItemMeta) -> Result<()> {
    conn.execute("UPDATE items SET meta = ?2 WHERE id = ?1", params![id, crypto::seal(key, &serde_json::to_vec(m)?)])?;
    Ok(())
}

fn set_meta_group(conn: &Connection, k: &Keys, id: i64, group: Option<i64>) -> Result<()> {
    let (_, key, mut m) = read_meta(conn, k, id)?;
    if m.group != group {
        m.group = group;
        write_meta(conn, &key, id, &m)?;
    }
    Ok(())
}

fn rekey(conn: &Connection, k: &Keys, id: i64, lock_all: bool) -> Result<()> {
    let (locked, old, meta) = read_meta(conn, k, id)?;
    let want = lock_all || meta.own_lock || group_locked(conn, meta.group)?;
    if want == locked {
        return Ok(());
    }
    let (pinned, last_used, salt, blobs): (bool, i64, Vec<u8>, [Option<Vec<u8>>; 5]) = conn.query_row("SELECT pinned, last_used, salt, plain, html, rtf, png, thumb FROM items WHERE id = ?1", [id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, [r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?])))?;
    let mut opened: [Option<Vec<u8>>; 5] = Default::default();
    for (o, b) in opened.iter_mut().zip(blobs) {
        *o = open_blob(&old, b)?;
    }
    let mut tag = dedup_tag(k, want, opened[0].as_deref(), opened[3].as_deref())?;
    if let Some(t) = &tag
        && let Some((other, other_used, other_salt)) = conn.query_row("SELECT id, last_used, salt FROM items WHERE dedup = ?1", [t], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get::<_, Vec<u8>>(2)?))).optional()?
    {
        if !pinned {
            if last_used > other_used {
                let (_, _, kept) = read_meta(conn, k, other)?;
                let newer = ItemMeta { group: kept.group, own_lock: kept.own_lock, ..meta };
                let key = row_key(k, want, &other_salt)?;
                let [plain, html, rtf, png, thumb] = opened.map(|b| b.map(|b| crypto::seal(&key, &b)));
                let meta = crypto::seal(&key, &serde_json::to_vec(&newer)?);
                conn.execute("UPDATE items SET last_used = ?2, enc_v = ?3, meta = ?4, plain = ?5, html = ?6, rtf = ?7, png = ?8, thumb = ?9 WHERE id = ?1", params![other, last_used, ENC_V1, meta, plain, html, rtf, png, thumb])?;
            }
            conn.execute("DELETE FROM items WHERE id = ?1", [id])?;
            return Ok(());
        }
        tag = None;
    }
    let key = row_key(k, want, &salt)?;
    let [plain, html, rtf, png, thumb] = opened.map(|b| b.map(|b| crypto::seal(&key, &b)));
    let meta = crypto::seal(&key, &serde_json::to_vec(&meta)?);
    conn.execute("UPDATE items SET locked = ?2, dedup = ?3, enc_v = ?4, meta = ?5, plain = ?6, html = ?7, rtf = ?8, png = ?9, thumb = ?10 WHERE id = ?1", params![id, want, tag, ENC_V1, meta, plain, html, rtf, png, thumb])?;
    Ok(())
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        Self::init(Connection::open(path)?, Some(path.with_extension("vacuum")))
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?, None)
    }

    fn init(conn: Connection, vacuum_stamp: Option<PathBuf>) -> Result<Self> {
        conn.pragma_update(None, "secure_delete", "ON")?;
        let version: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
        if version < 1 {
            conn.execute_batch(SCHEMA_V1)?;
        }
        if version < 2 {
            conn.execute_batch(SCHEMA_V2)?;
        }
        Ok(Store { conn, lock_all: false, vacuum_stamp })
    }

    pub fn upsert_clip(&mut self, k: &Keys, c: &NewClip, now: i64) -> Result<i64> {
        let f = &c.formats;
        let locked = self.lock_all && k.lock_root.is_some();
        let tag = dedup_tag(k, locked, f.plain.as_deref().map(str::as_bytes), f.png.as_deref())?;
        let existing: Option<ExistingRow> = match &tag {
            Some(t) => self.conn.query_row("SELECT id, salt, meta, html IS NOT NULL, rtf IS NOT NULL, png IS NOT NULL FROM items WHERE dedup = ?1", [t], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, [r.get(3)?, r.get(4)?, r.get(5)?]))).optional()?,
            None => None,
        };
        let (salt, group, own_lock, stored) = match &existing {
            Some((_, salt, meta, [html, rtf, png])) => {
                let m = open_meta(&row_key(k, locked, salt)?, meta)?;
                let subset = (f.html.is_none() || *html) && (f.rtf.is_none() || *rtf) && (f.png.is_none() || *png);
                (salt.clone(), m.group, m.own_lock, subset.then_some(m))
            }
            None => (crypto::new_salt().to_vec(), None, false, None),
        };
        let key = row_key(k, locked, &salt)?;
        let size = match (&f.plain, &f.png) {
            (Some(p), _) => p.chars().count() as u64,
            (None, Some(png)) => png.len() as u64,
            (None, None) => 0,
        };
        let kind = stored.as_ref().map_or(c.kind, |m| m.kind);
        let has_image = f.png.is_some() || stored.as_ref().is_some_and(|m| m.has_image);
        let meta = ItemMeta { kind, source: c.source.clone(), size, preview: f.plain.as_deref().unwrap_or("").chars().take(PREVIEW_CHARS).collect(), group, has_image, own_lock };
        let meta = crypto::seal(&key, &serde_json::to_vec(&meta)?);
        let seal = |b: Option<&[u8]>| b.map(|b| crypto::seal(&key, b));
        let (plain, html, rtf, png, thumb) = (seal(f.plain.as_deref().map(str::as_bytes)), seal(f.html.as_deref()), seal(f.rtf.as_deref()), seal(f.png.as_deref()), seal(c.thumb.as_deref()));
        match existing {
            Some((id, ..)) => {
                let sql = if stored.is_some() {
                    "UPDATE items SET last_used = ?2, enc_v = ?3, meta = ?4, plain = COALESCE(?5, plain), html = COALESCE(?6, html), rtf = COALESCE(?7, rtf), png = COALESCE(?8, png), thumb = COALESCE(?9, thumb) WHERE id = ?1"
                } else {
                    "UPDATE items SET last_used = ?2, enc_v = ?3, meta = ?4, plain = ?5, html = ?6, rtf = ?7, png = ?8, thumb = ?9 WHERE id = ?1"
                };
                self.conn.execute(sql, params![id, now, ENC_V1, meta, plain, html, rtf, png, thumb])?;
                Ok(id)
            }
            None => {
                self.conn.execute("INSERT INTO items (last_used, locked, dedup, enc_v, salt, meta, plain, html, rtf, png, thumb) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)", params![now, locked, tag, ENC_V1, salt, meta, plain, html, rtf, png, thumb])?;
                Ok(self.conn.last_insert_rowid())
            }
        }
    }

    pub fn load_all(&self, k: &Keys) -> Result<Vec<Item>> {
        Ok(self.load(k, &[])?.items)
    }

    pub fn load(&self, k: &Keys, prev: &[Item]) -> Result<Loaded> {
        let groups: Vec<(i64, bool)> = self.conn.prepare("SELECT id, locked FROM groups")?.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<rusqlite::Result<_>>()?;
        let rows: Vec<ItemRow> = self
            .conn
            .prepare("SELECT id, last_used, pinned, locked, salt, meta, substr(plain, 1, 12) FROM items ORDER BY last_used DESC, id DESC")?
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?)))?
            .collect::<rusqlite::Result<_>>()?;
        let mut out = Loaded::default();
        for (id, last_used, pinned, locked, salt, meta, text_tag) in rows {
            let key = match row_key(k, locked, &salt) {
                Err(StoreError::Locked) => {
                    let locked_by = self.lock_all.then_some(LockSource::All);
                    out.items.push(Item { id, last_used, pinned, locked, meta: None, search: None, locked_by, text_tag: None });
                    continue;
                }
                key => key?,
            };
            let cached = prev.iter().find(|i| i.id == id && i.text_tag.is_some() && i.text_tag == text_tag).and_then(|i| i.search.clone());
            match self.open_row(&key, id, &meta, cached) {
                Ok((mut meta, search)) => {
                    meta.group = meta.group.filter(|g| groups.iter().any(|(id, _)| id == g));
                    let in_locked_group = groups.iter().any(|&(g, l)| l && meta.group == Some(g));
                    let locked_by = if !locked { None } else if self.lock_all { Some(LockSource::All) } else if in_locked_group { Some(LockSource::Group) } else { None };
                    out.locked_ok += usize::from(locked);
                    out.items.push(Item { id, last_used, pinned, locked, meta: Some(meta), search, locked_by, text_tag });
                }
                Err(StoreError::Sql(e)) => return Err(e.into()),
                Err(_) => {
                    out.unreadable += 1;
                    out.locked_bad += usize::from(locked);
                }
            }
        }
        Ok(out)
    }

    fn open_row(&self, key: &ItemKey, id: i64, meta: &[u8], cached: Option<Arc<str>>) -> Result<(ItemMeta, Option<Arc<str>>)> {
        let meta = open_meta(key, meta)?;
        if cached.is_some() {
            return Ok((meta, cached));
        }
        let plain: Option<Vec<u8>> = self.conn.query_row("SELECT plain FROM items WHERE id = ?1", [id], |r| r.get(0))?;
        Ok((meta, open_text(key, plain)?.map(|p| search_text(&p))))
    }

    pub fn formats(&self, k: &Keys, id: i64) -> Result<ClipFormats> {
        let (locked, salt, plain, html, rtf, png): (bool, Vec<u8>, _, _, _, _) = self.conn.query_row("SELECT locked, salt, plain, html, rtf, png FROM items WHERE id = ?1", [id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)))?;
        let key = row_key(k, locked, &salt)?;
        Ok(ClipFormats { plain: open_text(&key, plain)?, html: open_blob(&key, html)?, rtf: open_blob(&key, rtf)?, png: open_blob(&key, png)? })
    }

    pub fn thumb(&self, k: &Keys, id: i64) -> Result<Option<Vec<u8>>> {
        let (locked, salt, thumb): (bool, Vec<u8>, _) = self.conn.query_row("SELECT locked, salt, thumb FROM items WHERE id = ?1", [id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
        open_blob(&row_key(k, locked, &salt)?, thumb)
    }

    pub fn touch(&mut self, id: i64, now: i64) -> Result<()> {
        self.conn.execute("UPDATE items SET last_used = ?2 WHERE id = ?1", [id, now])?;
        Ok(())
    }

    pub fn set_pinned(&mut self, k: &Keys, id: i64, on: bool) -> Result<()> {
        let lock_all = self.lock_all;
        let tx = self.conn.transaction()?;
        if !on {
            set_meta_group(&tx, k, id, None)?;
        }
        tx.execute("UPDATE items SET pinned = ?2 WHERE id = ?1", params![id, on])?;
        if !on {
            rekey(&tx, k, id, lock_all)?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn pin_items(&mut self, ids: &[i64]) -> Result<()> {
        let tx = self.conn.transaction()?;
        for id in ids {
            tx.execute("UPDATE items SET pinned = 1 WHERE id = ?1", [id])?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn set_group(&mut self, k: &Keys, item: i64, group: Option<i64>) -> Result<()> {
        let lock_all = self.lock_all;
        let tx = self.conn.transaction()?;
        set_meta_group(&tx, k, item, group)?;
        if group.is_some() {
            tx.execute("UPDATE items SET pinned = 1 WHERE id = ?1", [item])?;
        }
        rekey(&tx, k, item, lock_all)?;
        tx.commit()?;
        Ok(())
    }

    pub fn set_item_locked(&mut self, k: &Keys, id: i64, on: bool) -> Result<()> {
        if k.lock_root.is_none() {
            return Err(StoreError::Locked);
        }
        let lock_all = self.lock_all;
        let tx = self.conn.transaction()?;
        let (_, key, mut m) = read_meta(&tx, k, id)?;
        m.own_lock = on;
        write_meta(&tx, &key, id, &m)?;
        rekey(&tx, k, id, lock_all)?;
        tx.commit()?;
        Ok(())
    }

    pub fn set_group_locked(&mut self, k: &Keys, id: i64, on: bool) -> Result<()> {
        if k.lock_root.is_none() {
            return Err(StoreError::Locked);
        }
        let members = self.members(k, id)?;
        let lock_all = self.lock_all;
        let tx = self.conn.transaction()?;
        tx.execute("UPDATE groups SET locked = ?2 WHERE id = ?1", params![id, on])?;
        for item in members {
            rekey(&tx, k, item, lock_all)?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn reconcile_all(&mut self, k: &Keys) -> Result<()> {
        let lock_all = self.lock_all;
        let tx = self.conn.transaction()?;
        let ids: Vec<i64> = tx.prepare("SELECT id FROM items ORDER BY last_used DESC, id DESC")?.query_map([], |r| r.get(0))?.collect::<rusqlite::Result<_>>()?;
        for id in ids {
            match rekey(&tx, k, id, lock_all) {
                Err(StoreError::Crypto(_) | StoreError::Json(_)) => {}
                r => r?,
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn has_unsealed(&self) -> Result<bool> {
        Ok(self.lock_all && self.conn.query_row("SELECT EXISTS(SELECT 1 FROM items WHERE locked = 0)", [], |r| r.get(0))?)
    }

    pub fn has_locked(&self) -> Result<bool> {
        Ok(self.conn.query_row("SELECT EXISTS(SELECT 1 FROM items WHERE locked = 1) OR EXISTS(SELECT 1 FROM groups WHERE locked = 1)", [], |r| r.get(0))?)
    }

    fn members(&self, k: &Keys, group: i64) -> Result<Vec<i64>> {
        Ok(self.load_all(k)?.into_iter().filter(|i| i.meta.as_ref().is_some_and(|m| m.group == Some(group))).map(|i| i.id).collect())
    }

    pub fn create_group(&mut self, k: &Keys, name: &str) -> Result<i64> {
        let salt = crypto::new_salt();
        let name = crypto::seal(&row_key(k, false, &salt)?, name.as_bytes());
        self.conn.execute("INSERT INTO groups (enc_v, salt, name) VALUES (?1, ?2, ?3)", params![ENC_V1, salt.to_vec(), name])?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn rename_group(&mut self, k: &Keys, id: i64, name: &str) -> Result<()> {
        let salt: Vec<u8> = self.conn.query_row("SELECT salt FROM groups WHERE id = ?1", [id], |r| r.get(0))?;
        let name = crypto::seal(&row_key(k, false, &salt)?, name.as_bytes());
        self.conn.execute("UPDATE groups SET enc_v = ?2, name = ?3 WHERE id = ?1", params![id, ENC_V1, name])?;
        Ok(())
    }

    pub fn delete_group(&mut self, k: &Keys, id: i64) -> Result<()> {
        if k.lock_root.is_none() && group_locked(&self.conn, Some(id))? {
            return Err(StoreError::Locked);
        }
        let members = self.members(k, id)?;
        let lock_all = self.lock_all;
        let tx = self.conn.transaction()?;
        for item in &members {
            set_meta_group(&tx, k, *item, None)?;
        }
        tx.execute("DELETE FROM groups WHERE id = ?1", [id])?;
        for item in members {
            rekey(&tx, k, item, lock_all)?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn list_groups(&self, k: &Keys) -> Result<Vec<Group>> {
        let mut st = self.conn.prepare("SELECT id, locked, salt, name FROM groups ORDER BY id")?;
        let rows = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get::<_, Vec<u8>>(2)?, r.get::<_, Vec<u8>>(3)?)))?;
        rows.map(|row| {
            let (id, locked, salt, name) = row?;
            let name = open_text(&row_key(k, false, &salt)?, Some(name))?.unwrap_or_default();
            Ok(Group { id, name, locked })
        })
        .collect()
    }

    pub fn apply_retention(&mut self, max_items: u32, max_days: u32, now: i64) -> Result<usize> {
        let cutoff = now - i64::from(max_days) * DAY;
        let aged = self.conn.execute("DELETE FROM items WHERE pinned = 0 AND last_used < ?1", [cutoff])?;
        let over = self.conn.execute("DELETE FROM items WHERE pinned = 0 AND id NOT IN (SELECT id FROM items WHERE pinned = 0 ORDER BY last_used DESC, id DESC LIMIT ?1)", [max_items])?;
        Ok(aged + over)
    }

    pub fn delete_item(&mut self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM items WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn delete_history(&mut self) -> Result<()> {
        self.conn.execute("DELETE FROM items WHERE pinned = 0", [])?;
        Ok(())
    }

    pub fn delete_all(&mut self) -> Result<()> {
        self.conn.execute_batch("BEGIN; DELETE FROM items; DELETE FROM groups; COMMIT;")?;
        Ok(())
    }

    pub fn vacuum_due(&self, now: i64) -> bool {
        let last = self.vacuum_stamp.as_ref().and_then(|p| std::fs::read_to_string(p).ok()).and_then(|s| s.trim().parse::<i64>().ok());
        last.is_none_or(|t| now - t >= DAY)
    }

    pub fn vacuum(&self, now: i64) -> Result<()> {
        self.conn.execute_batch("VACUUM;")?;
        if let Some(p) = &self.vacuum_stamp
            && let Err(e) = std::fs::write(p, now.to_string())
        {
            eprintln!("vacuum stamp: {e}");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{Keys, MasterKey};

    fn clip(t: &str) -> NewClip {
        NewClip { formats: ClipFormats { plain: Some(t.into()), ..Default::default() }, kind: Kind::Text, source: "notepad.exe".into(), thumb: None }
    }

    #[test]
    fn dedup_moves_to_top_and_replaces_formats() {
        let mut s = Store::open_in_memory().unwrap();
        let k = Keys::new(MasterKey::generate());
        let a = s.upsert_clip(&k, &clip("x"), 100).unwrap();
        s.upsert_clip(&k, &clip("y"), 200).unwrap();
        let mut again = clip("x");
        again.formats.html = Some(b"<b>x</b>".to_vec());
        again.source = "chrome.exe".into();
        assert_eq!(s.upsert_clip(&k, &again, 300).unwrap(), a);
        let items = s.load_all(&k).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!((items[0].id, items[0].last_used), (a, 300));
        assert_eq!(items[0].meta.as_ref().unwrap().source, "chrome.exe");
        assert_eq!(s.formats(&k, a).unwrap().html.as_deref(), Some(&b"<b>x</b>"[..]));
    }

    #[test]
    fn retention_by_count_keeps_pins() {
        let mut s = Store::open_in_memory().unwrap();
        let k = Keys::new(MasterKey::generate());
        let first = s.upsert_clip(&k, &clip("0"), 0).unwrap();
        s.set_pinned(&k, first, true).unwrap();
        for i in 1..=600 { s.upsert_clip(&k, &clip(&i.to_string()), i).unwrap(); }
        s.apply_retention(500, 30, 600).unwrap();
        let items = s.load_all(&k).unwrap();
        assert_eq!(items.len(), 501);
        assert!(items.iter().any(|i| i.id == first));
    }

    #[test]
    fn retention_by_age_keeps_pins() {
        let day = 86_400;
        let mut s = Store::open_in_memory().unwrap();
        let k = Keys::new(MasterKey::generate());
        let old = s.upsert_clip(&k, &clip("old"), 0).unwrap();
        let pinned = s.upsert_clip(&k, &clip("pinned"), 0).unwrap();
        s.set_pinned(&k, pinned, true).unwrap();
        let fresh = s.upsert_clip(&k, &clip("fresh"), 31 * day).unwrap();
        assert_eq!(s.apply_retention(500, 30, 31 * day).unwrap(), 1);
        let ids: Vec<i64> = s.load_all(&k).unwrap().iter().map(|i| i.id).collect();
        assert!(!ids.contains(&old));
        assert!(ids.contains(&pinned) && ids.contains(&fresh));
    }

    #[test]
    fn nothing_plain_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("k.db");
        {
            let mut s = Store::open(&path).unwrap();
            let k = Keys::new(MasterKey::generate());
            let mut c = clip("GIZLI-ICERIK-123");
            c.source = "gizli-kaynak.exe".into();
            s.upsert_clip(&k, &c, 1).unwrap();
        }
        let bytes = std::fs::read(&path).unwrap();
        let hay = String::from_utf8_lossy(&bytes);
        assert!(!hay.contains("GIZLI-ICERIK-123"));
        assert!(!hay.contains("gizli-kaynak.exe"));
    }

    #[test]
    fn delete_history_keeps_pins() {
        let mut s = Store::open_in_memory().unwrap();
        let k = Keys::new(MasterKey::generate());
        let p = s.upsert_clip(&k, &clip("p"), 1).unwrap();
        s.set_pinned(&k, p, true).unwrap();
        s.upsert_clip(&k, &clip("h"), 2).unwrap();
        s.delete_history().unwrap();
        assert_eq!(s.load_all(&k).unwrap().len(), 1);
        s.delete_all().unwrap();
        assert!(s.load_all(&k).unwrap().is_empty());
    }

    #[test]
    fn delete_history_keeps_grouped_and_drops_locked_unpinned() {
        let mut s = Store::open_in_memory().unwrap();
        let mut k = Keys::new(MasterKey::generate());
        k.lock_root = Some(crypto::lock_root(&k.master, b"imza"));
        let g = s.create_group(&k, "is").unwrap();
        let grouped = s.upsert_clip(&k, &clip("grouped"), 1).unwrap();
        s.set_group(&k, grouped, Some(g)).unwrap();
        let pinned = s.upsert_clip(&k, &clip("pinned"), 2).unwrap();
        s.set_pinned(&k, pinned, true).unwrap();
        let locked = s.upsert_clip(&k, &clip("locked"), 3).unwrap();
        s.set_item_locked(&k, locked, true).unwrap();
        s.upsert_clip(&k, &clip("loose"), 4).unwrap();
        k.lock_root = None;
        s.delete_history().unwrap();
        let mut left: Vec<_> = s.load_all(&k).unwrap().into_iter().map(|i| i.id).collect();
        left.sort_unstable();
        assert_eq!(left, [grouped, pinned]);
        assert_eq!(s.list_groups(&k).unwrap().len(), 1);
    }

    #[test]
    fn locked_item_can_be_deleted_without_its_key() {
        let mut s = Store::open_in_memory().unwrap();
        let mut k = Keys::new(MasterKey::generate());
        k.lock_root = Some(crypto::lock_root(&k.master, b"imza"));
        let gone = s.upsert_clip(&k, &clip("gizli"), 1).unwrap();
        s.set_item_locked(&k, gone, true).unwrap();
        s.upsert_clip(&k, &clip("kalan"), 2).unwrap();
        k.lock_root = None;
        s.delete_item(gone).unwrap();
        assert_eq!(s.load_all(&k).unwrap().iter().map(|i| i.search.as_deref()).collect::<Vec<_>>(), [Some("kalan")]);
    }

    #[test]
    fn group_lifecycle() {
        let mut s = Store::open_in_memory().unwrap();
        let k = Keys::new(MasterKey::generate());
        let id = s.upsert_clip(&k, &clip("x"), 1).unwrap();
        let g = s.create_group(&k, "SQL parçaları").unwrap();
        s.set_group(&k, id, Some(g)).unwrap();
        let item = s.load_all(&k).unwrap().remove(0);
        assert!(item.pinned);
        assert_eq!(item.meta.unwrap().group, Some(g));
        s.rename_group(&k, g, "SQL").unwrap();
        assert_eq!(s.list_groups(&k).unwrap()[0].name, "SQL");
        s.delete_group(&k, g).unwrap();
        let item = s.load_all(&k).unwrap().remove(0);
        assert!(item.pinned);
        assert_eq!(item.meta.unwrap().group, None);
        assert!(s.list_groups(&k).unwrap().is_empty());
    }

    #[test]
    fn pin_items_pins_only_the_given_ids_locked_ones_included_without_the_root() {
        let mut s = Store::open_in_memory().unwrap();
        let k = unlocked_keys();
        let a = s.upsert_clip(&k, &clip("a"), 1).unwrap();
        let b = s.upsert_clip(&k, &clip("b"), 2).unwrap();
        let c = s.upsert_clip(&k, &clip("c"), 3).unwrap();
        s.set_item_locked(&k, b, true).unwrap();
        s.pin_items(&[a, b]).unwrap();
        let pinned = |id| s.load_all(&locked_keys()).unwrap().into_iter().find(|i| i.id == id).unwrap().pinned;
        assert!(pinned(a) && pinned(b) && !pinned(c));
    }

    #[test]
    fn unpin_leaves_group() {
        let mut s = Store::open_in_memory().unwrap();
        let k = Keys::new(MasterKey::generate());
        let id = s.upsert_clip(&k, &clip("x"), 1).unwrap();
        let g = s.create_group(&k, "g").unwrap();
        s.set_group(&k, id, Some(g)).unwrap();
        s.set_pinned(&k, id, false).unwrap();
        assert_eq!(s.load_all(&k).unwrap()[0].meta.as_ref().unwrap().group, None);
    }

    fn unlocked_keys() -> Keys {
        let mut k = Keys::new(MasterKey::from_bytes(&[2u8; 64]));
        k.lock_root = Some(crate::crypto::lock_root(&k.master, b"imza"));
        k
    }
    fn locked_keys() -> Keys { Keys::new(MasterKey::from_bytes(&[2u8; 64])) }

    #[test]
    fn locked_item_hidden_without_root() {
        let mut s = Store::open_in_memory().unwrap();
        let id = s.upsert_clip(&unlocked_keys(), &clip("gizli"), 1).unwrap();
        s.set_item_locked(&unlocked_keys(), id, true).unwrap();
        let hidden = s.load_all(&locked_keys()).unwrap().remove(0);
        assert!(hidden.locked && hidden.meta.is_none() && hidden.search.is_none());
        assert!(s.formats(&locked_keys(), id).is_err());
        assert_eq!(s.load_all(&unlocked_keys()).unwrap()[0].search.as_deref(), Some("gizli"));
    }

    #[test]
    fn locking_requires_root_and_does_not_pin() {
        let mut s = Store::open_in_memory().unwrap();
        let id = s.upsert_clip(&locked_keys(), &clip("x"), 1).unwrap();
        assert!(s.set_item_locked(&locked_keys(), id, true).is_err());
        s.set_item_locked(&unlocked_keys(), id, true).unwrap();
        assert!(!s.load_all(&unlocked_keys()).unwrap()[0].pinned);
        assert_eq!(s.apply_retention(0, 30, 2).unwrap(), 1);
    }

    #[test]
    fn locked_item_not_deduped() {
        let mut s = Store::open_in_memory().unwrap();
        let k = unlocked_keys();
        let a = s.upsert_clip(&k, &clip("x"), 1).unwrap();
        s.set_item_locked(&k, a, true).unwrap();
        assert_ne!(s.upsert_clip(&k, &clip("x"), 2).unwrap(), a);
    }

    #[test]
    fn locked_group_hides_members_and_new_members() {
        let mut s = Store::open_in_memory().unwrap();
        let k = unlocked_keys();
        let a = s.upsert_clip(&k, &clip("a"), 1).unwrap();
        let b = s.upsert_clip(&k, &clip("b"), 2).unwrap();
        let g = s.create_group(&k, "kasa").unwrap();
        s.set_group(&k, a, Some(g)).unwrap();
        s.set_group_locked(&k, g, true).unwrap();
        s.set_group(&k, b, Some(g)).unwrap();
        assert!(s.load_all(&locked_keys()).unwrap().iter().all(|i| i.meta.is_none()));
        assert_eq!(s.list_groups(&locked_keys()).unwrap()[0].name, "kasa");
        s.set_group_locked(&k, g, false).unwrap();
        assert!(s.load_all(&locked_keys()).unwrap().iter().all(|i| i.meta.is_some()));
    }

    #[test]
    fn unlocking_item_restores_it_and_its_dedup() {
        let mut s = Store::open_in_memory().unwrap();
        let k = unlocked_keys();
        let a = s.upsert_clip(&k, &clip("x"), 1).unwrap();
        s.set_item_locked(&k, a, true).unwrap();
        s.set_item_locked(&k, a, false).unwrap();
        assert_eq!(s.load_all(&locked_keys()).unwrap()[0].search.as_deref(), Some("x"));
        assert_eq!(s.upsert_clip(&locked_keys(), &clip("x"), 2).unwrap(), a);
    }

    #[test]
    fn leaving_locked_group_needs_root_and_unlocks_the_item() {
        let mut s = Store::open_in_memory().unwrap();
        let k = unlocked_keys();
        let a = s.upsert_clip(&k, &clip("a"), 1).unwrap();
        let g = s.create_group(&k, "kasa").unwrap();
        s.set_group(&k, a, Some(g)).unwrap();
        s.set_group_locked(&k, g, true).unwrap();
        assert_eq!(s.load_all(&k).unwrap()[0].locked_by, Some(LockSource::Group));
        assert!(s.set_pinned(&locked_keys(), a, false).is_err());
        s.set_pinned(&k, a, false).unwrap();
        let item = s.load_all(&locked_keys()).unwrap().remove(0);
        assert!(!item.locked && item.meta.is_some() && item.locked_by.is_none());
    }

    #[test]
    fn leaving_locked_group_under_lock_all_stays_locked() {
        let mut s = Store::open_in_memory().unwrap();
        s.lock_all = true;
        let k = unlocked_keys();
        let a = s.upsert_clip(&k, &clip("a"), 1).unwrap();
        let g = s.create_group(&k, "kasa").unwrap();
        s.set_group(&k, a, Some(g)).unwrap();
        s.set_group_locked(&k, g, true).unwrap();
        s.set_pinned(&k, a, false).unwrap();
        let item = s.load_all(&k).unwrap().remove(0);
        assert!(item.locked);
        assert_eq!(item.locked_by, Some(LockSource::All));
    }

    #[test]
    fn own_lock_is_not_inherited() {
        let mut s = Store::open_in_memory().unwrap();
        let k = unlocked_keys();
        let a = s.upsert_clip(&k, &clip("a"), 1).unwrap();
        s.set_item_locked(&k, a, true).unwrap();
        let item = s.load_all(&k).unwrap().remove(0);
        assert!(item.locked && item.locked_by.is_none());
    }

    #[test]
    fn plain_recopy_keeps_stored_rich_formats() {
        let mut s = Store::open_in_memory().unwrap();
        let k = Keys::new(MasterKey::generate());
        let mut rich = clip("x");
        rich.formats.html = Some(b"<b>x</b>".to_vec());
        rich.formats.rtf = Some(b"{\rtf1 x}".to_vec());
        let id = s.upsert_clip(&k, &rich, 1).unwrap();
        assert_eq!(s.upsert_clip(&k, &clip("x"), 2).unwrap(), id);
        let f = s.formats(&k, id).unwrap();
        assert_eq!((f.html.as_deref(), f.rtf.as_deref()), (Some(&b"<b>x</b>"[..]), Some(&b"{\rtf1 x}"[..])));
        assert_eq!(s.load_all(&k).unwrap()[0].last_used, 2);
    }

    #[test]
    fn lock_all_migration_keeps_the_newer_copy() {
        let mut s = Store::open_in_memory().unwrap();
        s.lock_all = true;
        let k = unlocked_keys();
        let mut old = clip("x");
        old.formats.html = Some(b"<i>eski</i>".to_vec());
        let kept = s.upsert_clip(&k, &old, 1).unwrap();
        let mut new = clip("x");
        new.formats.html = Some(b"<b>yeni</b>".to_vec());
        new.source = "chrome.exe".into();
        s.upsert_clip(&locked_keys(), &new, 3).unwrap();
        s.reconcile_all(&k).unwrap();
        let items = s.load_all(&k).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!((items[0].id, items[0].last_used, items[0].locked), (kept, 3, true));
        assert_eq!(items[0].meta.as_ref().unwrap().source, "chrome.exe");
        assert_eq!(s.formats(&k, kept).unwrap().html.as_deref(), Some(&b"<b>yeni</b>"[..]));
    }

    #[test]
    fn pending_copies_are_reported_until_sealed() {
        let mut s = Store::open_in_memory().unwrap();
        s.lock_all = true;
        s.upsert_clip(&locked_keys(), &clip("y"), 1).unwrap();
        assert!(s.has_unsealed().unwrap());
        s.reconcile_all(&unlocked_keys()).unwrap();
        assert!(!s.has_unsealed().unwrap());
    }

    #[test]
    fn undecryptable_row_is_skipped_and_counted() {
        let mut s = Store::open_in_memory().unwrap();
        let k = unlocked_keys();
        s.upsert_clip(&k, &clip("a"), 1).unwrap();
        let bad = s.upsert_clip(&k, &clip("b"), 2).unwrap();
        s.conn.execute("UPDATE items SET meta = ?2 WHERE id = ?1", params![bad, vec![0u8; 40]]).unwrap();
        let l = s.load(&k, &[]).unwrap();
        assert_eq!((l.items.len(), l.unreadable), (1, 1));
        s.reconcile_all(&k).unwrap();
    }

    #[test]
    fn search_text_is_capped_folded_and_reused() {
        let mut s = Store::open_in_memory().unwrap();
        let k = Keys::new(MasterKey::generate());
        s.upsert_clip(&k, &clip(&format!("İSTANBUL {}", "ş".repeat(60_000))), 1).unwrap();
        let first = s.load(&k, &[]).unwrap().items;
        let search = first[0].search.clone().unwrap();
        assert!(search.len() <= 64 * 1024 && search.starts_with("istanbul"));
        let again = s.load(&k, &first).unwrap().items;
        assert!(std::sync::Arc::ptr_eq(&search, again[0].search.as_ref().unwrap()));
    }

    #[test]
    fn vacuum_time_survives_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("v.db");
        let s = Store::open(&path).unwrap();
        assert!(s.vacuum_due(1_000_000));
        s.vacuum(1_000_000).unwrap();
        drop(s);
        let s = Store::open(&path).unwrap();
        assert!(!s.vacuum_due(1_000_000 + 3_600));
        assert!(s.vacuum_due(1_000_000 + 86_401));
    }

    #[test]
    fn deleting_locked_group_requires_root_and_unlocks_members() {
        let mut s = Store::open_in_memory().unwrap();
        let k = unlocked_keys();
        let a = s.upsert_clip(&k, &clip("a"), 1).unwrap();
        let g = s.create_group(&k, "kasa").unwrap();
        s.set_group(&k, a, Some(g)).unwrap();
        s.set_group_locked(&k, g, true).unwrap();
        assert!(s.delete_group(&locked_keys(), g).is_err());
        s.delete_group(&k, g).unwrap();
        let item = s.load_all(&k).unwrap().remove(0);
        assert!(!item.locked && item.pinned);
        assert_eq!(item.meta.unwrap().group, None);
    }

    #[test]
    fn deleted_group_id_is_never_reused() {
        let mut s = Store::open_in_memory().unwrap();
        let k = unlocked_keys();
        let a = s.upsert_clip(&k, &clip("a"), 1).unwrap();
        let g = s.create_group(&k, "eski").unwrap();
        s.set_group(&k, a, Some(g)).unwrap();
        s.set_item_locked(&k, a, true).unwrap();
        s.delete_group(&locked_keys(), g).unwrap();
        let fresh = s.create_group(&k, "yeni").unwrap();
        assert_ne!(fresh, g);
        assert_eq!(s.load_all(&k).unwrap()[0].meta.as_ref().unwrap().group, None);
    }

    #[test]
    fn lock_all_seals_new_copies_under_root_and_migrates_pending_ones() {
        let mut s = Store::open_in_memory().unwrap();
        s.lock_all = true;
        let k = unlocked_keys();
        let kept = s.upsert_clip(&k, &clip("x"), 1).unwrap();
        let pending = s.upsert_clip(&locked_keys(), &clip("y"), 2).unwrap();
        let again = s.upsert_clip(&locked_keys(), &clip("x"), 3).unwrap();
        assert_eq!(s.load_all(&locked_keys()).unwrap().iter().filter(|i| i.meta.is_some()).count(), 2);
        s.reconcile_all(&k).unwrap();
        let items = s.load_all(&locked_keys()).unwrap();
        assert!(items.iter().all(|i| i.locked && i.meta.is_none()));
        assert_eq!(items.iter().map(|i| (i.id, i.last_used)).collect::<Vec<_>>(), [(kept, 3), (pending, 2)]);
        assert_ne!(again, kept);
        assert_eq!(s.upsert_clip(&k, &clip("y"), 4).unwrap(), pending);
    }

    #[test]
    fn v1_database_keeps_groups_after_upgrade() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("v1.db");
        let k = Keys::new(MasterKey::generate());
        let salt = crypto::new_salt();
        Connection::open(&path).unwrap().execute_batch(SCHEMA_V1).unwrap();
        Connection::open(&path).unwrap().execute("INSERT INTO groups (id, enc_v, salt, name) VALUES (7, 1, ?1, ?2)", params![salt.to_vec(), crypto::seal(&row_key(&k, false, &salt).unwrap(), b"eski")]).unwrap();
        let mut s = Store::open(&path).unwrap();
        assert_eq!(s.list_groups(&k).unwrap()[0].id, 7);
        s.delete_group(&k, 7).unwrap();
        assert_ne!(s.create_group(&k, "yeni").unwrap(), 7);
    }

    #[test]
    fn group_name_not_plain_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("g.db");
        {
            let mut s = Store::open(&path).unwrap();
            s.create_group(&Keys::new(MasterKey::generate()), "GIZLI-GRUP").unwrap();
        }
        assert!(!String::from_utf8_lossy(&std::fs::read(&path).unwrap()).contains("GIZLI-GRUP"));
    }
}
