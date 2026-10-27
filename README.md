# Keyforge

Configurador nativo de teclados en Rust + egui/eframe. Incluye las **fases 1 y 2** y el primer bloque de **identificación guiada con dispositivos simulados**. Funciona sin teclado y sin servicios de red; la primera compilación necesita descargar dependencias. El nombre del producto sigue siendo Keyforge (el documento de requisitos utiliza también «QMK Studio»).

## Ejecutar

Instala Rust estable con rustup. En Windows 10/11 se recomienda el toolchain MSVC y Visual Studio Build Tools con «Desarrollo para el escritorio con C++» y Windows SDK. Alternativamente, GNU necesita un enlazador MinGW-w64 en PATH.

```sh
cargo run --locked
```

En este equipo se ha preparado Rust + MinGW de forma aislada en `.tools/` (ignorado por Git). Puedes ejecutar `./dev.ps1` desde PowerShell sin modificar el PATH del sistema. El ejecutable de desarrollo está en `target/debug/keyforge.exe`. El script también acepta `check`, `test`, `build`, `clippy` y `fmt`.

En Arch Linux: `sudo pacman -S --needed base-devel rustup libxkbcommon libx11 libxcursor libxi libxrandr wayland mesa`, seguido de `rustup default stable`. En Debian/Ubuntu: compilador C, pkg-config y bibliotecas de desarrollo de X11, Wayland y xkbcommon (`build-essential pkg-config libx11-dev libxcursor-dev libxi-dev libxrandr-dev libxkbcommon-dev libwayland-dev libegl1-mesa-dev`). El backend seleccionado es OpenGL/glow; necesita un controlador gráfico compatible. La compatibilidad Linux se verifica también mediante el workflow de CI cuando se ejecute en GitHub.

La ventana ofrece tarjetas visuales: **Corne 42, Corne 36 y Planck 48**. Pulsa «Ver este teclado» para explorar una distribución. El selector de demostración permite probar coincidencia USB, confirmación simulada, datos incompletos, dispositivo desconocido y desconexión. Todas las conexiones actuales son simuladas. La selección manual solo cambia la vista local.

En «Información avanzada» puedes cargar un JSON por ruta absoluta o relativa al directorio de trabajo. Los errores mantienen el último teclado válido. La lectura se realiza en un hilo de trabajo. Usa el control de zoom, arrastra el lienzo y selecciona teclas para inspeccionarlas. No necesitas saber el microcontrolador ni el bootloader.

## Implementado

- Ventana nativa, tema oscuro, panel de inspección y modo demostración sin hardware.
- Layout JSON versionado, deserialización estricta y validación semántica.
- Motor genérico con posiciones, tamaños, rotación sobre el centro, ajuste a ventana, zoom y desplazamiento.
- Ejemplo Corne de 42 teclas, 21 por mitad, seis pulgares y dos pantallas ilustrativas.
- Tests de geometría, validación, lectura y serialización.
- Catálogo local versionado de QMK con fuentes fijadas a un commit; datos y geometría de Corne rev1 y Planck rev6.
- Servicios de descubrimiento simulado, identificación, capacidades, pantallas y bloqueo de flasheo independientes de egui.
- Estados de identificación sin porcentajes: confirmado por adaptador, coincidencia USB de alta confianza, probable y desconocido.
- Capacidades independientes: comprobado, según metadatos, no disponible y desconocido. Los metadatos no habilitan acciones físicas.

Los layouts del catálogo proceden de QMK. El ejemplo independiente `layouts/corne.json` representa la distribución habitual 3×6+3, sin corresponder a una revisión de PCB verificada. Las etiquetas muestran coordenadas de matriz, **no keycodes**. Sus IPS son ilustrativas y su resolución es `null`. El catálogo menciona OLED para Corne, pero no deduce que tu Corne tenga OLED o IPS ni cuántas pantallas incorpora. No hay pantallas dibujadas en los layouts del catálogo porque sus dimensiones físicas no están documentadas en estas fuentes.

## Arquitectura

```text
src/main.rs                   arranque de eframe
src/app.rs                    coordinación y estado de la interfaz
src/ui/keyboard.rs            dibujo genérico y selección
src/domain/layout.rs          modelo, geometría y validación sin egui
src/domain/devices.rs         modelos, evidencias y estados de capacidades
src/services/                descubrimiento, identificación, sesión y capacidades
src/infrastructure/catalog.rs catálogo QMK validado
src/infrastructure/layouts.rs lectura limitada, errores y JSON
catalog/qmk.json              catálogo offline con procedencia
layouts/corne.json            definición declarativa de ejemplo
tests/layouts.rs              validación y carga de extremo a extremo
docs/architecture.md          decisiones y evolución
```

No se crean módulos vacíos para funcionalidades futuras. Las siguientes fases incorporarán capas/keycodes/perfiles al dominio, servicios de edición con historial y traits para adaptadores de teclado y pantalla, sin acoplar USB a egui.

## Formato del layout (v1)

El archivo de ejemplo documenta todos los campos. `matrix` es `[filas,columnas]`, las posiciones de teclas son índices desde cero y no pueden repetirse. `halves` admite uno o varios grupos con nombres libres. `geometry` utiliza unidades de tecla: x/y son la esquina superior izquierda antes de rotar; width/height son dimensiones positivas; rotation son grados horarios alrededor del centro. `variant` identifica la variante física del archivo; cada variante se guarda por ahora como un layout independiente.

`key_count` debe coincidir con las teclas. Se rechazan versiones desconocidas, campos desconocidos, IDs repetidos, matrices fuera de rango, referencias a mitades inexistentes, geometrías no finitas y pantallas inválidas. Límite de archivo: 2 MiB; máximo 1024 teclas y 16 pantallas. `usb` y `firmware` son metadatos declarativos, nunca una prueba de capacidades negociadas. El esquema no modela aún teclas de contorno irregular ni capacidades de edición del firmware.

## Verificación

```sh
cargo fmt --check
cargo check --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Verificación local realizada en Windows: fmt, check y clippy con `-D warnings` correctos; doce tests correctos; ejecutable compilado. Las pruebas cubren geometría, archivos inválidos, identificación ambigua, evidencia de otro dispositivo, separación de simulación/hardware, metadatos de pantallas y desconexión. El entorno de archivos puede emitir un aviso de caché incremental por no poder crear hardlinks; Rust copia los archivos y la compilación termina. Linux tiene un workflow preparado; su resultado debe consultarse en Actions.

## Pendiente

Siguiente fase: edición local de comportamientos y capas, historial y perfiles con exportación. Después: HID real y protocolos comprobados, asistente gráfico de herramientas QMK, compilación/flasheo asistidos y composición/exportación IPS. No hay lectura/escritura USB, flasheo, instalación automática de herramientas, edición de asignaciones ni transmisión de imágenes en esta versión. La edición local futura no dependerá de haber identificado el hardware.
