use crate::classify::Kind;
use crate::store::Item;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GroupFilter {
    All,
    Pinned,
    Group { id: i64 },
}

#[derive(Deserialize, Clone)]
pub struct Query {
    pub text: String,
    pub kind: Option<Kind>,
    pub group: GroupFilter,
    pub source: Option<String>,
}

pub fn fold(s: &str) -> String {
    s.to_lowercase().replace('\u{307}', "")
}

pub fn filter<'a>(items: &'a [Item], q: &Query) -> Vec<&'a Item> {
    let needle = fold(&q.text);
    items
        .iter()
        .filter(|i| match q.group {
            GroupFilter::All => true,
            GroupFilter::Pinned => i.pinned,
            GroupFilter::Group { id } => i.meta.as_ref().is_some_and(|m| m.group == Some(id)),
        })
        .filter(|i| {
            let Some(m) = &i.meta else { return needle.is_empty() && q.kind.is_none() && q.source.is_none() };
            q.kind.is_none_or(|k| k == m.kind) && q.source.as_deref().is_none_or(|s| m.source.eq_ignore_ascii_case(s)) && (i.search.as_deref().unwrap_or("").contains(&needle) || fold(&m.source).contains(&needle))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{Item, ItemMeta};

    fn it(id: i64, text: &str, kind: Kind, pinned: bool, group: Option<i64>) -> Item {
        Item { id, last_used: id, pinned, locked: false, search: Some(fold(text).into()), locked_by: None, text_tag: None, meta: Some(ItemMeta { kind, source: "code.exe".into(), size: 1, preview: text.into(), group, has_image: false, own_lock: false }) }
    }
    fn q(text: &str) -> Query { Query { text: text.into(), kind: None, group: GroupFilter::All, source: None } }
    fn from(mut item: Item, source: &str) -> Item {
        item.meta.as_mut().unwrap().source = source.into();
        item
    }
    fn ids(v: Vec<&Item>) -> Vec<i64> { v.iter().map(|i| i.id).collect() }

    #[test]
    fn case_insensitive_text_and_source() {
        let items = vec![it(1, "select 1", Kind::Code, false, None), it(2, "merhaba", Kind::Text, false, None)];
        assert_eq!(ids(filter(&items, &q("SELECT"))), vec![1]);
        assert_eq!(ids(filter(&items, &q("code.exe"))), vec![1, 2]);
    }

    #[test]
    fn turkish_dotted_i_matches() {
        let items = vec![it(1, "İstanbul ofisi", Kind::Text, false, None)];
        assert_eq!(ids(filter(&items, &q("istanbul"))), vec![1]);
        assert_eq!(ids(filter(&items, &q("ISTANBUL"))), vec![1]);
    }

    #[test]
    fn kind_and_group_filters() {
        let items = vec![it(1, "a", Kind::Code, true, Some(7)), it(2, "b", Kind::Text, true, None), it(3, "c", Kind::Text, false, None)];
        assert_eq!(ids(filter(&items, &Query { kind: Some(Kind::Text), ..q("") })), vec![2, 3]);
        assert_eq!(ids(filter(&items, &Query { group: GroupFilter::Pinned, ..q("") })), vec![1, 2]);
        assert_eq!(ids(filter(&items, &Query { group: GroupFilter::Group { id: 7 }, ..q("") })), vec![1]);
    }

    #[test]
    fn source_filter_matches_the_exe_ignoring_case_and_combines_with_the_rest() {
        let hidden = Item { id: 5, last_used: 5, pinned: false, locked: true, search: None, locked_by: None, text_tag: None, meta: None };
        let items = vec![it(1, "select 1", Kind::Code, true, None), from(it(2, "merhaba", Kind::Text, false, None), "Floorp.exe"), it(3, "selam", Kind::Text, false, None), from(it(4, "select 2", Kind::Code, false, None), "CODE.EXE"), hidden];
        let by = |source: &str| Query { source: Some(source.into()), ..q("") };
        assert_eq!(ids(filter(&items, &by("code.exe"))), vec![1, 3, 4]);
        assert_eq!(ids(filter(&items, &by("floorp.exe"))), vec![2]);
        assert_eq!(ids(filter(&items, &Query { text: "select".into(), ..by("code.exe") })), vec![1, 4]);
        assert_eq!(ids(filter(&items, &Query { kind: Some(Kind::Text), ..by("code.exe") })), vec![3]);
        assert_eq!(ids(filter(&items, &Query { group: GroupFilter::Pinned, ..by("code.exe") })), vec![1]);
        assert!(filter(&items, &by("notepad.exe")).is_empty());
        assert_eq!(ids(filter(&items, &q(""))), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn hidden_locked_items_only_without_filters() {
        let items = vec![Item { id: 1, last_used: 1, pinned: false, locked: true, search: None, locked_by: None, text_tag: None, meta: None }];
        assert_eq!(ids(filter(&items, &q(""))), vec![1]);
        assert!(filter(&items, &q("a")).is_empty());
    }
}
