The date-group and Pinned header in item lists: Pinler, Bugün, Dün, Bu hafta, Daha eski.

**Anatomy.** `control-strong` in `text-muted`, padding `space-6` top, `space-3` inline (aligned with the ring room). Pinler takes a 12px `pin` icon. An optional count sits on the right in `control`. In the List layout every date group (not Pinler) ends with a 16px ghost `pin` button, and right-clicking the header opens a menu with the same action: "Tümünü pinle / Pin all" pins every item listed in that group under the current search and filters, locked items included, without confirmation.

**Rules.** The order is always Pinler → Bugün → Dün → Bu hafta → Daha eski. Hide empty groups. Headers are not focusable and not selectable. Arrow keys skip them.

| TR | EN |
|---|---|
| Pinler | Pinned |
| Bugün · Dün · Bu hafta · Daha eski | Today · Yesterday · This week · Older |
| Tümünü pinle | Pin all |
