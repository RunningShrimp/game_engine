# 编辑器 API 参考

## EditorContext

编辑器上下文，管理编辑器 UI。

### 示例

```rust
use game_engine::editor::EditorContext;

let mut editor_ctx = EditorContext::new(&window, &device, format);

// 处理事件
if editor_ctx.handle_event(&window, &event) {
    // 事件已处理
}

// 开始帧
editor_ctx.begin_frame(&window);

// 渲染 UI
inspect_world_ui(&editor_ctx.context, &mut world);

// 结束帧
let (shapes, renderer) = editor_ctx.end_frame(&window);
```

## ShortcutManager

快捷键管理器。

### 示例

```rust
use game_engine::editor::{ShortcutManager, ShortcutAction, Modifiers};

let mut shortcuts = ShortcutManager::new();

// 注册动作
shortcuts.register_action(ShortcutAction::Undo, Box::new(|| {
    command_manager.undo();
}));

// 处理输入
shortcuts.handle_input(Modifiers::ctrl(), "Z");
```

## AssetBrowser

资源浏览器。

### 示例

```rust
use game_engine::editor::AssetBrowser;

let mut browser = AssetBrowser::new("./assets");

// 渲染浏览器
browser.render(&mut ui);

// 获取选中的资源
if let Some(index) = browser.selected_asset {
    let asset = &browser.assets[index];
    println!("Selected: {}", asset.name);
}
```

## SceneEditor

场景编辑器。

### 示例

```rust
use game_engine::editor::SceneEditor;

let mut editor = SceneEditor::new();

// 渲染编辑器
editor.render(&mut ui, &mut world);

// 获取选中的实体
if let Some(entity) = editor.selected_entity {
    // 处理选中的实体
}
```

