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
(`public`/`private`), `TAG`, `REMOTE`, `GIT_NAME`, `GIT_EMAIL`.

## Releases

Her `v*` etiketinde GitHub Actions, Linux (`tar.gz`) ve Windows (`zip`) icin calistirilabilir
paket uretir. Paketlerin icinde `swiftx` yurutulebilir dosyasi ve `resources/` klasoru bulunur.

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
