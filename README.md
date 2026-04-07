# glossa-morpho

一个基于 **Tauri 2 + Nuxt 4 + Nuxt UI 4 + Nuxt i18n** 的本地化字符串编辑器骨架工程。

这个代码包的目标不是直接实现完所有功能，而是把以下内容一次性搭好：

- 桌面壳与前端工作台布局
- 项目标签页、三栏编辑器、Treemap/列表双视图
- 前端状态管理和 Tauri 命令调用边界
- Rust 侧命令骨架与内存版示例数据
- 项目格式、解析器、导出器、数据库迁移目录占位
- 夹具、测试目录和实现 TODO

## 已经包含的可开工内容

- Nuxt 4 `app/` 结构
- Nuxt UI 4 全局 `UApp` 容器
- Nuxt i18n 应用语言切换
- Pinia 状态管理
- Tauri 2 命令骨架
- 模拟项目加载 / 创建 / 保存 / 批量替换 / Treemap
- 浏览器开发模式下的 mock fallback

## 建议开发顺序

1. 先跑通前端布局与交互
2. 把 Rust 内存示例替换为 SQLite + 项目目录读写
3. 先实现 JSON / YAML / properties 的 importer/exporter
4. 再补历史记录、候选、批量编辑、正则编辑
5. 最后补 RESX / XAML / XLIFF round-trip

## 运行

### 仅跑前端

```bash
npm install
npm run dev
```

### 跑 Tauri 桌面壳

```bash
npm install
npm run tauri:dev
```

### 生产构建

```bash
npm install
npm run tauri:build
```

## 当前实现说明

当前 Rust 侧是 **可交互的内存版骨架**：

- 打开项目 / 新建项目：返回示例数据
- 保存项目：返回保存时间
- 更新条目：修改内存状态
- 批量替换：按搜索条件批量改写目标值
- Treemap：按 key 分组输出简单节点

这足够你开始写：

- 真正的项目目录格式
- SQLite schema
- importer/exporter
- 文件系统与回写流程
- 历史与候选的真实落库

## 目录提示

- `app/`：Nuxt 前端
- `shared/`：共享类型与常量
- `src-tauri/`：桌面端 Rust
- `tests/fixtures/`：导入导出测试夹具

## 下一步最建议先做

先从这 3 个文件开始：

- `src-tauri/src/commands/project.rs`
- `src-tauri/src/commands/entry.rs`
- `src-tauri/src/parsers/json.rs`

然后把 `mockDesktopApi` 的返回值逐步替换成真实命令结果。
