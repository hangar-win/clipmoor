The custom-colour editor. The user sets nine roles per mode, and everything else is derived.

**Anatomy.** A Koyu/Açık SegmentedControl picks the mode being edited. Two groups follow, **Yüzey** (Zemin `ground`, Kart `card`, Yazı `text`, Seçim `selection`) and **Tür renkleri** (Metin, Resim, Link, Kod, Kilitli). Each 44px row has a 24px swatch (`radius-item`, `swatch-edge` plus a `settings-field` ring), the role name, a hex field (`code`, `control-sm`), and a live contrast readout. The footnote says the derived roles are computed.

**Contrast readout.** Card is checked against Zemin, Yazı against Zemin (≥4.5), Seçim against Zemin (≥3, or "+ kenar" when the light `selection-edge` covers it), and hues against Zemin and Kart (≥3). A failure shows in `settings-danger` with an icon and the ratio. The value is clamped on apply (see Theming in the brand book), never silently.

| TR | EN |
|---|---|
| Yüzey · Zemin · Kart · Yazı · Seçim | Surface · Ground · Card · Text · Selection |
| Tür renkleri | Type colours |
| Çizgi, şerit, hover ve ikincil yazı bu renklerden hesaplanır. | Lines, strip, hover and secondary text are computed from these colours. |
