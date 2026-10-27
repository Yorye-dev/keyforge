# Keyforge

Configurador nativo de teclados en Rust + egui/eframe para Windows y Linux. Incluye exploración de layouts, catálogo QMK offline, identificación simulada, **editor local de teclas y capas** y **perfiles JSON**. Funciona sin teclado ni servicios cloud; la primera compilación descarga dependencias.

## Ejecutar

Instala Rust estable. En Windows 10/11 se recomienda MSVC con Visual Studio Build Tools (C++ y Windows SDK); GNU necesita MinGW-w64.

```sh
cargo run --locked
```

En este equipo, Rust y MinGW están aislados en `.tools/` (ignorado por Git). Desde PowerShell en `C:\Users\yorye\Proyectos\keyforge`:

```powershell
.\dev.ps1
```

El script acepta también `check`, `test`, `build`, `clippy` y `fmt`. El ejecutable de desarrollo está en `target/debug/keyforge.exe`.

Arch Linux: `sudo pacman -S --needed base-devel rustup libxkbcommon libx11 libxcursor libxi libxrandr wayland mesa`, después `rustup default stable`. Debian/Ubuntu: `build-essential pkg-config libx11-dev libxcursor-dev libxi-dev libxrandr-dev libxkbcommon-dev libwayland-dev libegl1-mesa-dev`. Los diálogos usan XDG Desktop Portal (con el backend de tu escritorio y D-Bus); rfd utiliza Zenity como alternativa. La aplicación no instala estos componentes automáticamente. El renderizado utiliza OpenGL/glow.

## Editar un perfil

1. Arranca con un Corne vacío o pulsa **Cambiar teclado** y elige Corne 42, Corne 36 o Planck 48.
2. Pulsa una tecla y elige el tipo de comportamiento en el panel derecho.
3. Configura sus parámetros y pulsa **Asignar a esta tecla**.
4. Crea, duplica o renombra capas. Las nuevas empiezan transparentes; las duplicadas conservan las asignaciones.
5. Usa **Copiar / Pegar**, **Deshacer / Rehacer** y **Guardar perfil…**.

Los perfiles nuevos empiezan sin asignaciones: no se ha leído el mapa del teclado físico. Los borradores del panel solo se incorporan al perfil al pulsar **Asignar**. `TRNS` significa transparente y `—` significa sin acción. Un borde ámbar indica cambios respecto al último guardado. El número máximo es de 32 capas **locales**; no implica que el firmware admita ese número.

Comportamientos: letras A–Z, números, F1–F24, teclas habituales, modificadores izquierdo/derecho, combinaciones con modificadores, multimedia, capas momentáneas/alternables, mod-tap y layer-tap. GUI equivale a Windows/Super. Los símbolos dependen del idioma configurado en el sistema operativo. Las funciones duales se representan localmente; sus tiempos y compatibilidad real dependen de un futuro adaptador de firmware.

No se puede eliminar la última capa ni una capa referenciada desde otra. Cambiar una asignación después de deshacer abre una nueva rama del historial y descarta rehacer. El historial conserva hasta 100 operaciones durante la sesión.

Atajos sin un campo de texto activo: **Ctrl+Z**, **Ctrl+Y / Ctrl+Shift+Z**, **Ctrl+S**. Copiar y pegar asignaciones utiliza un portapapeles interno con botones; no modifica el portapapeles del sistema.

## Guardar y abrir

**Guardar perfil…** permite elegir un archivo JSON; **Abrir perfil** lo valida antes de reemplazar el trabajo actual. Cada archivo incluye nombre, layout completo, capas y comportamientos. Puedes conservar varios perfiles como archivos distintos y compartirlos sin depender del catálogo instalado.

Los cambios sin guardar provocan un aviso al cerrar, abrir otro perfil o cambiar de layout. Puedes guardar, descartar o cancelar. Si cancelas el diálogo o falla la operación, el trabajo sigue en memoria. No hay autoguardado ni recuperación después de un cierre forzado. Guardar un perfil no lo aplica al teclado.

La escritura utiliza un archivo temporal en el mismo directorio y reemplazo atómico; se valida antes de tocar el archivo destino. Los diálogos y el acceso al disco se ejecutan en un worker. La edición queda temporalmente deshabilitada para evitar que cambie el perfil mientras se guarda.

## Catálogo y detección

Catálogo local de Corne rev1 y Planck rev6 normalizado desde una revisión fija oficial de QMK, con fuentes en `catalog/qmk.json`. No es un catálogo completo ni un importador general de QMK.

En **Demostración de detección** se pueden probar coincidencia USB, confirmación simulada, datos incompletos, teclado desconocido y desconexión. No hay HID real. La selección visual de un modelo no confirma el dispositivo ni autoriza una actualización.

Las capacidades distinguen comprobado, según metadatos, no disponible y desconocido. Solo la edición local de teclas y capas está habilitada. OLED/iluminación en el catálogo son información de firmware, no detección de hardware. El ejemplo IPS de **Información avanzada** tiene resolución desconocida y pantallas ilustrativas.

## Arquitectura

```text
src/app.rs                     coordinación, navegación y cambios sin guardar
src/domain/layout.rs           geometría y validación del layout
src/domain/keymap.rs           comportamientos, capas y perfil versionado
src/domain/devices.rs          metadatos, evidencias y capacidades
src/services/editor.rs         operaciones transaccionales, historial y portapapeles
src/services/file_jobs.rs      diálogos y acceso a archivos en worker
src/services/                  identificación, descubrimiento simulado y capacidades
src/infrastructure/profiles.rs importación validada y escritura atómica
src/infrastructure/layouts.rs  carga JSON con límite de tamaño
src/infrastructure/catalog.rs  catálogo QMK offline
src/ui/editor.rs               formulario de comportamientos y capas
src/ui/keyboard.rs             dibujo genérico con asignaciones y cambios
tests/                         layouts, identificación, editor y persistencia
```

El dominio no depende de egui ni USB. El renderizador recibe etiquetas y cambios, sin conocer protocolos. La identificación no condiciona las operaciones locales. Consulta `docs/architecture.md` para decisiones y contratos futuros.

## Formatos

Layout v1: `matrix` es [filas, columnas] con índices desde cero. `geometry` usa unidades de tecla, x/y de esquina superior izquierda y rotación horaria sobre el centro. Cada variante física es un layout independiente. Máximo 1024 teclas, 16 pantallas y 2 MiB por archivo de layout.

Perfil v1: `schema_version`, `name`, `layout` y `layers`. Cada capa tiene un ID estable, nombre y un mapa de asignaciones indexado por IDs lógicos de tecla. Debe incluir exactamente las teclas del layout y todos los destinos de capa deben existir. Máximo 8 MiB, 32 capas y 48 caracteres en nombres. Se rechazan versiones desconocidas, campos desconocidos y datos semánticamente inválidos. El historial, selección activa y portapapeles no se serializan.

## Verificación

```sh
cargo fmt --check
cargo check --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

24 tests: geometría, layouts, catálogo, identificación y desconexión, referencias de capas, rollback, historial, portapapeles, límites, serialización y conservación del archivo anterior ante errores. Revisión manual de asignación, duplicado, aviso al cerrar y guardado desde el diálogo nativo en Windows. La base anterior pasó CI Windows/Linux; esta rama debe pasar el mismo workflow al publicarse.

## Siguientes fases

Biblioteca de perfiles y preferencias, ampliación de comportamientos y exportación a formatos de firmware, detección HID real, negociación de protocolos y lectura/escritura con vista previa/copia de seguridad/verificación. Después, asistente QMK para compilación/flasheo y composición/exportación IPS. No hay escritura USB, flasheo ni transmisión de imágenes en esta entrega.
