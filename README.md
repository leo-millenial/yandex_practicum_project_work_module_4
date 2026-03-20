# Image FFI Processor

Проект демонстрирует использование FFI и unsafe кода в Rust для динамической загрузки плагинов обработки изображений.

## Примеры

### Исходное изображение

![Original](logo.png)

### Mirror — отражение по горизонтали

```bash
cargo run --package image_processor -- \
    logo.png logo_mirror.png target/debug/libmirror_plugin.dylib
```

![Mirror](logo_mirror.png)

### Blur — размытие (радиус 3)

```bash
cargo run --package image_processor -- \
    logo.png logo_blur.png target/debug/libblur_plugin.dylib "3"
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
    <input.png> <output.png> <plugin.dylib> [params]
```

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

Пример в `mirror_plugin/src/lib.rs` и `blur_plugin/src/lib.rs`.
