# SwiftX Browser

Hizli, guvenli ve acik kaynakli web tarayicisi. Baslangicta Python + PySide6 (QtWebEngine)
ile yazildi; **v0.3.0** ile birlikte tamamen **Rust**'a ([tao](https://crates.io/crates/tao) +
[wry](https://crates.io/crates/wry)) tasindi.

## Ozellikler

- **Coklu sekme** + sekme uyutma (hafiza tasarrufu) ve renkli sekmeler
- **Navigasyon**: geri/ileri/yenile/ana sayfa, adres cubugu + arama
- **Yer imleri**: cubuk, ekleme/kaldirma, JSON ice/disa aktarma
- **Gecmis** (son 500 kayit) ve **indirmeler** yan panelde
- **Reklam/izleyici engelleyici** (navigasyon + sayfa ici)
- **Smooth Scroll**, **Auto Dark Mode**, **Sayfa icinde arama**, **Gelistirici araclari**
- **Oturum geri yukleme**, tam ekran, izin yonetimi (kamera/mikrofon/konum/...)
- **Eklenti sistemi**: Dark Reader, No Ads, Page Speed, Privacy Guard, Read Mode
- **Ayar kaliciligi** (`settings.json`)

## Klavye Kisayollari

| Kisayol | Islev |
| --- | --- |
| `Ctrl+T` / `Ctrl+W` | Yeni sekme / sekmeyi kapat |
| `Ctrl+L` | Adres cubuguna odaklan |
| `Ctrl+D` | Yer imine ekle |
| `Ctrl+H` / `Ctrl+J` | Gecmis / Indirmeler |
| `Ctrl+B` | Kenar cubugu |
| `Ctrl+R` / `F5` | Yenile |
| `Ctrl+F` | Sayfada ara |
| `Ctrl+S` | Yer imlerini disa aktar |
| `Ctrl+ +/-/0` | Zoom |
| `Alt+Sol/Sag` | Geri / Ileri |
| `F11` / `F12` | Tam ekran / Gelistirici araclari |

## Derleme

```bash
cd rust
cargo build --release
./target/release/swiftx
```

### Linux bagimliliklari

- `webkit2gtk-4.1` ve `gtk+-3.0` gelistirme paketleri
- Debian/Ubuntu: `sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev`
- Arch: `sudo pacman -S webkit2gtk-4.1 gtk3`
- Fedora: `sudo dnf install webkit2gtk4.1-devel gtk3-devel`

> **Wayland notu:** wry'nin Linux arka ucu alt-web gorunumleri icin X11 kullanir. Uygulama
> baslangicta `GDK_BACKEND=x11` secerek XWayland uzerinden calisir. Saf Wayland ortaminda
> XWayland kurulu olmalidir.

### Windows

WebView2 Runtime (Windows 10/11'de varsayilan gelir) gereklidir.

## Kaynaktan dagitim

```bash
GITHUB_TOKEN=ghp_xxxxxxxx ./setup.sh
```

Betik `gh` (GitHub CLI) gerektirmez. Bir Personal Access Token ile calisir
(<https://github.com/settings/tokens/new?scopes=repo,workflow>, scope: `repo` + `workflow`):
Rust'i (gerekirse) kurar, release
derlemesini yapar, git deposunu baslatir, GitHub deposunu olusturur, push eder ve `v0.3.0`
etiketini atarak release is akisini tetikler. Token yalnizca push aninda kullanilir ve
`.git/config` icinde saklanmaz.

`gh` kurulu ve oturum acilmissa token olmadan da calisir.

Ortam degiskenleri: `GITHUB_TOKEN`, `GITHUB_USER`, `REPO_NAME`, `VISIBILITY`
(`public`/`private`), `TAG`, `REMOTE`, `GIT_NAME`, `GIT_EMAIL`, `FORCE_TAG`.

Ayni etiketi (orn. halihazirda var olan `v0.3.0`) yeni bir derlemeye tasimak icin:

```bash
GITHUB_TOKEN=... FORCE_TAG=1 ./setup.sh
```

## Releases

Her `v*` etiketi push edildiginde **veya** GitHub'da bir release yayinlandiginda
(`release: published`) GitHub Actions calisir ve:

- Linux icin `swiftx-linux-x86_64.tar.gz` + ham `swiftx`
- Windows icin `swiftx-windows-x86_64.zip` + ham `swiftx.exe`

dosyalarini release'e yukler. Paketlerin icinde calistirilabilir dosya ve `resources/`
klasoru bulunur.

### Yeni surum nasil yayinlanir

1. `rust/Cargo.toml` ve `rust/src/core/constants.rs` icindeki surumu guncelle (orn. `0.3.1`).
2. `GITHUB_TOKEN=... TAG=v0.3.1 ./setup.sh` calistir (veya elle `git tag v0.3.1 && git push origin v0.3.1`).
3. Actions, release'i ve guncel ikili dosyalari otomatik olusturur.

## Otomatik Guncelleme

Uygulama acilista GitHub'daki **en son release**'i kontrol eder
(`/repos/Stegizm/SwiftX/releases/latest`). Daha yeni bir surum varsa bildirim gosterir;
**Ayarlar > Guncellemeler** bolumunden "Yeniden baslat ve guncelle" ile yeni ikili dosya
indirilir, uygulama kapanir, dosya degistirilir ve otomatik yeniden baslatilir.

Gereksinim: yeni surumun normal (draft/prerelease olmayan) bir release olarak ve
ilgili platformun ham ikili dosyasi (`swiftx` / `swiftx.exe`) eklenmis olmasi. Bu,
yukaridaki release akisiyla otomatik saglanir.

## Proje Yapisi

```
rust/
  src/
    main.rs          # giris noktasi / olay dongusu
    app.rs           # koordinator (eski MainWindow)
    ipc.rs           # chrome <-> Rust mesaj protokolu
    core/            # sabitler, modeller, storage, servisler
    engine/          # web gorunumu, reklam engelleme, scriptler
    ui/              # gomulu chrome arayuzu (HTML/CSS/JS)
  resources/         # ana sayfa + gorseller
setup.sh
.github/workflows/release.yml
```

## Lisans

MIT - bkz. [LICENSE](LICENSE).
