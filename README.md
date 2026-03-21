# Image FFI Processor

Проект демонстрирует использование FFI и unsafe кода в Rust для динамической загрузки плагинов обработки изображений.

## Примеры

### Mirror — отражение по горизонтали

```bash
cargo run --package image_processor -- \
    logo.png logo_mirror.png target/debug/libmirror_plugin.dylib /dev/null
```

![Mirror](logo_mirror.png)

### Blur — размытие (радиус 3)

```bash
echo '3' > params.txt
cargo run --package image_processor -- \
    logo.png logo_blur.png target/debug/libblur_plugin.dylib params.txt
```

![Blur](logo_blur.png)

## Сборка

```bash
cargo build --workspace
cargo build -p mirror_plugin
cargo build -p blur_plugin
```

## Использование

```bash
cargo run --package image_processor -- \
    <INPUT> <OUTPUT> <PLUGIN> <PARAMS_FILE>
```

Где:
- `<INPUT>` — путь к входному изображению
- `<OUTPUT>` — путь для сохранения результата
- `<PLUGIN>` — путь к файлу плагина (.so/.dylib)
- `<PARAMS_FILE>` — путь к файлу с параметрами для плагина

## Структура проекта

```
image_ffi_project/
├── Cargo.toml
├── image_processor/     # Основное приложение
├── mirror_plugin/       # Плагин отражения
├── blur_plugin/        # Плагин размытия
├── logo.png            # Исходное изображение
├── logo_mirror.png     # Результат mirror
└── logo_blur.png       # Результат blur
```

## Создание плагина

Плагин должен экспортировать:

```c
void process_image(uint32_t width, uint32_t height, uint8_t* rgba_data, const char* params);
```

Приложение читает содержимое файла `PARAMS_FILE` и передаёт его как строку в `params`.
Пример в `mirror_plugin/src/lib.rs` и `blur_plugin/src/lib.rs`.
