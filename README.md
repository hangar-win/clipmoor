# Clipmoor

Windows 10/11 (x64) için şifreli pano geçmişi. Win+V'nin yerini alır.

## Kurulum

1. `Clipmoor_<sürüm>_x64-setup.exe` dosyasını çalıştır. Kurulum yönetici izni istemez, yalnızca senin kullanıcı hesabına (`%LOCALAPPDATA%\Clipmoor`) kurulur.
2. Kurulum dosyası imzasız olduğu için Windows SmartScreen "Windows bilgisayarınızı korudu" uyarısı gösterir. **Ek bilgi**'ye, sonra **Yine de çalıştır**'a tıkla.
3. Son sayfadaki **Windows ile başla** kutusu varsayılan olarak işaretsizdir. İşaretlersen uygulama oturum açınca başlar. Bunu sonradan Ayarlar'dan değiştirebilirsin.

Bilinmesi gerekenler:

- **Akıllı Uygulama Denetimi** (Smart App Control) açık olan makinelerde imzasız uygulama çalışmaz.
- Yönetici olarak çalışan bir pencere öndeyken Win+V'ye basılırsa Windows'un kendi pano paneli açılır.
- Veriler yalnızca bu makinede, şifreli tutulur (`%LOCALAPPDATA%\app.clipmoor`). Anahtar kaybolursa (ör. TPM sıfırlanırsa ya da Windows yeniden kurulursa) geçmiş kurtarılamaz.
- Kaldırma, Windows ile başla kaydını da siler. Veriler varsayılan olarak kalır; silmek için kaldırma ekranındaki uygulama verilerini silme kutusunu işaretle.

## Installation

1. Run `Clipmoor_<version>_x64-setup.exe`. It does not ask for administrator rights and installs for your user account only (`%LOCALAPPDATA%\Clipmoor`).
2. The installer is unsigned, so Windows SmartScreen shows "Windows protected your PC". Click **More info**, then **Run anyway**.
3. The **Start with Windows** box on the last page is unchecked by default. Check it to start the app when you sign in. You can change this later in Settings.

Good to know:

- Unsigned apps do not run on machines with **Smart App Control** turned on.
- When a window running as administrator is in the foreground, Win+V opens the Windows clipboard panel instead.
- Data stays on this machine, encrypted (`%LOCALAPPDATA%\app.clipmoor`). If the key is lost (for example the TPM is reset or Windows is reinstalled), the history cannot be recovered.
- Uninstalling also removes the Start with Windows entry. Your data is kept by default; to delete it, check the delete application data box on the uninstall screen.
