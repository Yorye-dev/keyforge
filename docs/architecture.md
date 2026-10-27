# Decisiones y trabajo posterior

## Identificación guiada (nuevo alcance)

Se implementa primero la prioridad 10 de los requisitos nuevos: catálogo, modelos/capacidades, coincidencias, simulador y selección visual. El descubrimiento HID y los asistentes QMK permanecen en fases posteriores. `DeviceDiscoveryService` y `QmkMetadataRepository` son traits; `SimulatedDiscovery` y `LocalQmkCatalog` son sus implementaciones actuales. `KeyboardIdentificationService`, `DeviceCapabilitiesService`, `FirmwareCompatibilityService` y `DisplayCapabilitiesService` son servicios puros probados sin egui.

Una confirmación requiere evidencia aportada por el adaptador, ligada al ID del dispositivo y al origen real/simulado. No se deserializa desde JSON. Aún no hay adaptadores de protocolos reales; la confirmación ofrecida en pantalla es exclusivamente una prueba. Una única coincidencia VID/PID con nombres exactos de fabricante y producto produce alta confianza dentro del catálogo; no prueba firmware ni excluye modelos que todavía no están catalogados. VID/PID con datos incompletos o compartidos produce coincidencias probables; ninguna coincidencia produce desconocido. Respuestas de protocolo inconsistentes no se degradan silenciosamente a alta confianza USB.

La selección manual no modifica la identificación. Al cambiar o desconectar el simulador se descartan identidad y asociación de metadatos anteriores; el layout sigue visible para uso local. Capacidades de hardware y acciones implementadas son distintas: `actionable` permanece falso para todas las operaciones todavía pendientes. La validación de firmware bloquea siempre hasta disponer de comprobaciones de artefacto, modelo/revisión, destino, bootloader y adaptador.

`catalog/qmk.json` contiene solo dos modelos iniciales, tres variantes físicas y enlaces a una revisión fija oficial de QMK. No es un índice completo de QMK ni un importador general. Las coordenadas/matrices se normalizaron de esas fuentes; no se infiere la matriz desde el aspecto del teclado (Planck rev6 es físicamente 4×12 pero su matriz lógica es 8×6). El origen declara Pro Micro para Corne; no se convierte eso en una afirmación sobre el procesador conectado. OLED/iluminación son evidencias de catálogo, no hardware detectado. Los atributos de pantalla desconocidos se conservan como null.

La edición y los perfiles locales ya están implementados; el siguiente paso es ampliar la biblioteca de perfiles antes de conectar protocolos. El asistente de herramientas comprobará QMK CLI en un worker y pedirá consentimiento antes de instalaciones, permisos o privilegios. La actualización de firmware deberá confirmar modelo exacto, compatibilidad del archivo y dispositivo de destino con consentimiento explícito; nunca decidirá por VID/PID o por la tarjeta elegida.

## Base

Rust 2024, eframe/egui 0.37, serde/serde_json 1 y thiserror 2. Cargo.lock fija dependencias de la aplicación. Se usa la API actual `App::ui`; glow evita exigir Vulkan. La lógica de geometría no depende del framework gráfico. El cargador solo entrega layouts validados y limita lecturas. La UI conserva el layout anterior cuando falla una importación. La copia JSON incluida permite iniciar desde cualquier directorio, y la carga por ruta permite editar archivos externos.

La distribución física y las asignaciones son conceptos distintos. Los perfiles referencian IDs lógicos estables y cada capa contiene comportamientos estructurados, validados contra las teclas del layout. El historial guarda operaciones de edición locales; aplicar al dispositivo será una operación explícita independiente.

## Investigación de integraciones

- [QMK info.json](https://docs.qmk.fm/reference_info_json): define variantes de layouts, matriz y geometría. Un importador puede convertir x/y/w/h y matrix. Debe seleccionar una variante explícita, resolver metadatos heredados y transformar los pivotes rx/ry al convenio de rotación del modelo. Las pantallas y la división por mitades necesitarán información adicional; no se implementa importación automática en estas fases.
- [QMK Raw HID](https://docs.qmk.fm/features/rawhid): canal de transporte, no protocolo universal de configuración. En fase 5 se estudiará [hidapi](https://docs.rs/hidapi/latest/hidapi/) para enumeración y reportes, comprobando backend Linux, permisos udev y manejo de interfaces. No basta con reconocer VID/PID.
- [VIA](https://www.caniusevia.com/docs/specification/) necesita definición y firmware compatibles. El primer adaptador previsto negociará versión/capacidades y usará exclusivamente comandos documentados en la implementación oficial; el transporte HID por sí solo no permite inferirlos.
- [Vial](https://get.vial.today/docs/) requiere su port de firmware y contempla mecanismos de seguridad. Se implementará mediante otro adaptador, con límites negociados y desbloqueo explícito cuando proceda.
- [ZMK Studio](https://zmk.dev/docs/features/studio) usa soporte de firmware Studio; USB requiere su endpoint serial/RPC, no el adaptador HID de VIA. El número de capas y los comportamientos disponibles dependen del firmware.
- [Quantum Painter](https://docs.qmk.fm/quantum_painter) proporciona recursos y controladores gráficos en firmware QMK. No establece por sí solo un protocolo USB de carga de imágenes. Antes de la fase IPS se necesitan modelo/controlador, resolución, orientación, formato de color y firmware de cada mitad.

## Contratos futuros

`KeyboardBackend` abstraerá enumeración, conexión, capacidades, lectura, escritura y verificación. Un worker poseerá la conexión y publicará eventos tipados para evitar I/O USB en el hilo de UI. El dispositivo simulado implementará el mismo contrato y permitirá simular desconexiones y lecturas discrepantes. No se añade un trait especulativo sin operaciones implementadas en esta fase.

`DisplayBackend` distinguirá exportación local de transferencia soportada por hardware. La composición independiente por pantalla/capa pertenecerá al dominio; recursos y conversión al formato destino serán servicios. Las capacidades comprobadas vendrán del backend, nunca del nombre del modelo o de la mera presencia de una pantalla en JSON.

Lectura y escritura físicas requerirán coincidencia de matriz/layout, límites de capa, vista previa, copia local y confirmación explícita. Los errores de escritura parcial deben conservarse y comunicarse; no se afirmará que el dispositivo coincide hasta completar la verificación.


## Editor local y perfiles (fase 3 y base de fase 4)

`domain/keymap.rs` modela comportamientos como enums serializables, sin códigos binarios ni supuestas equivalencias de protocolo. Un perfil incluye su layout validado, 1–32 capas completas y referencias por ID estable. Las asignaciones iniciales están vacías: no se infiere un mapa del firmware a partir de las etiquetas QMK.

`KeymapEditor` valida cada modificación antes de incorporarla al historial. Ante un error restaura la instantánea anterior. El historial almacena hasta 100 instantáneas de capas/nombre/capa activa; el layout permanece fijo. Los IDs nuevos no se reutilizan durante la sesión, incluso después de eliminar/deshacer, para que el portapapeles no cambie silenciosamente de destino. Las capas referenciadas desde otras no se eliminan. Renombrar conserva IDs y destinos. El indicador de cambios compara contenido con la instantánea guardada, no solo un contador.

`FileJob` posee el worker para diálogos rfd y persistencia. Mientras existe un job la UI permite repintado pero bloquea edición, conservando una instantánea consistente para el guardado. Cancelación y errores mantienen el perfil anterior. Las transiciones de cierre/cambio de layout/apertura consultan cambios pendientes; guardar y continuar solo avanza si la escritura ha terminado correctamente.

`profiles::save` valida y serializa antes de escribir, usa tempfile en el directorio destino, sincroniza el contenido y persiste mediante reemplazo atómico. No incluye autoguardado ni recuperación ante terminación del proceso. La importación limita el tamaño y valida versión, layout, IDs y referencias. El perfil no puede declarar capacidades verificadas ni una identidad de dispositivo.

Las capacidades LocalKeys y LocalLayers ahora son Verified/actionable; Write, Flash, DynamicDisplays y las demás operaciones físicas siguen deshabilitadas. No se ha añadido ningún adaptador de hardware.
