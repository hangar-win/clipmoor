# Pano Yöneticisi

Windows'ta Win+V'nin yerini alan, odak çalmayan, şifreli pano geçmişi uygulaması.

## Language

### Öğeler

**Öğe (Item)**:
Panoya bir kez giren tek bir içerik ve onun meta bilgisi.
_Avoid_: kayıt, clip, entry

**Tür (Type)**:
Bir öğenin otomatik atanan sınıfı: metin, resim, link, renk, kod.
_Avoid_: kategori, format

**Kaynak uygulama (Source app)**:
Öğe panoya girdiği anda ön planda olan uygulama.
_Avoid_: owner, sahip

**Hızlı numara (Quick number)**:
Paneldeki görünür sırada ilk dokuz öğeye verilen 1–9 numarası.
_Avoid_: index, kısayol numarası

### Geçmiş ve pinler

**Geçmiş (History)**:
Pinlenmemiş öğelerin, saklama sınırına tabi listesi.
_Avoid_: log, arşiv

**Pin**:
Öğenin saklama sınırından muaf tutulması; pinli öğe asla otomatik silinmez.
_Avoid_: favori, yıldız

**Pin grubu (Pin group)**:
Kullanıcının adlandırdığı, pinli öğelerden oluşan grup.
_Avoid_: klasör, koleksiyon, board

**Kilitli grup (Locked group)**:
Açılması Windows Hello doğrulaması gerektiren pin grubu.
_Avoid_: gizli grup, kasa

**Kilitli öğe (Locked item)**:
Tek başına açılması Windows Hello doğrulaması gerektiren öğe; kilitlemek onu pinlemez.
_Avoid_: gizli öğe, korumalı öğe

### Etkileşim

**Panel**:
Açma kısayoluyla (varsayılan Win+V) imlecin yanında açılan, odak almayan geçmiş penceresi.
_Avoid_: popup, overlay, pencere

**Açık tutulan panel (Kept-open panel)**:
Kullanıcının isteğiyle dışarı tıklamada, uygulama değişiminde ve yapıştırmadan sonra kapanmayan Panel.
_Avoid_: sabitlenmiş panel, pinli panel

**Hedef uygulama (Target app)**:
Panel açılmadan önce kullanıcının yazdığı ve yapıştırmanın gideceği uygulama.
_Avoid_: aktif uygulama

**Yapıştır (Paste)**:
Öğeyi panoya yazıp hedef uygulamaya Ctrl+V göndermek.

**Düz yapıştır (Plain paste)**:
Öğeyi biçimi atılmış, yalnızca metin olarak yapıştırmak.
_Avoid_: biçimsiz yapıştır, plain text paste

**Kopyala (Copy)**:
Öğeyi yalnızca panoya yazmak, hedef uygulamaya dokunmamak.

**Kısayol (Shortcut)**:
Bir işi tetikleyen tuş kombinasyonu. Paneli açma, Yapıştır, Kopyala, Düz yapıştır, Pin, Önizleme, Kapat ve Hızlı numara değiştiricisi Ayarlar'dan değiştirilebilir; oklar, Home/End, Tab, Backspace ve yazı sabittir.
_Avoid_: hotkey, kısayol tuşu

**Açma kısayolu (Open shortcut)**:
Paneli her uygulamadan açan kısayol; varsayılanı Win+V.
_Avoid_: global hotkey, tetikleyici

**Ele geçirme (Takeover)**:
Uygulamanın Win+V kısayolunu Windows'tan önce yakalaması. Açma kısayolu Win+V değilse "Win+V'yi de yakala" olarak görünür ve kapalıyken Win+V Windows'a kalır.
_Avoid_: override, hijack
