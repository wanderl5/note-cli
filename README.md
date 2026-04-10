# note-cli

`note-cli` 是一个基于 Rust 的 Markdown 笔记静态站点生成器。  
它会扫描笔记目录，生成可导航的 HTML 站点，并内置代码高亮、分类页和全文搜索。

## 功能特性

- 将 `docs/`（或自定义目录）中的 `.md` 文件构建为静态网页
- 支持 YAML Frontmatter（如 `title`、`description`、`date`、`tags`）
- 自动生成首页、分类页、文章页与侧边导航
- 生成 `search.json`，前端支持关键词逻辑检索（`&&`、`||`、`!`、括号）
- 内置本地 HTTP 服务，支持预览构建结果
- 支持路径映射（例如中文目录映射为英文 URL）

## 环境要求

- Rust stable（建议通过 `rustup` 安装）
- Cargo（随 Rust 一起安装）

## 快速开始

### 1) 初始化配置

在项目目录执行：

```bash
cargo run -- init
```

会生成 `note.toml` 模板文件。

### 2) 编辑配置

示例 `note.toml`：

```toml
[site]
title = "My Notes"
base_url = "/"
docs_dir = "../docs"
dist_dir = "../dist"

[path_map]
"编程" = "coding"
"数据库" = "database"
```

- `site.title`：站点标题
- `site.docs_dir`：Markdown 源目录
- `site.dist_dir`：构建输出目录
- `path_map`：目录/文件名映射（去掉数字前缀后生效，如 `00.编程` -> `coding`）

### 3) 构建站点

```bash
cargo run -- build
```

默认读取当前目录下的 `note.toml`，并输出到 `dist_dir`。

### 4) 本地预览

```bash
cargo run -- serve --port 8080
```

访问：`http://localhost:8080/`

## 命令说明

```bash
note-cli [OPTIONS] <COMMAND>
```

### 全局参数

- `-c, --config <CONFIG>`：指定配置文件路径（默认：`note.toml`）

### 子命令

- `build`：构建静态站点
- `serve`：构建并启动本地服务（可指定 `--port`）
- `init`：在当前目录生成 `note.toml` 模板

## Frontmatter 示例

在 Markdown 文件头部可选添加：

```yaml
---
title: Assert
description: Java 断言语法与使用场景
date: 2026-04-10
tags:
  - Java
  - 编程
---
```

若未提供 `title`，默认使用文件名（去掉 `.md`）作为页面标题。

## 目录与 URL 规则

- 支持目录和文件名前缀排序：如 `00.编程/02.Java/Assert.md`
- 生成 URL 时会先去除数字前缀，再应用 `path_map`
- 上例可映射为：`/coding/java/Assert/`
- 会跳过：隐藏项（`.`、`@` 开头）、`index.md`、`索引.md`、`superpowers`

## 输出内容

构建完成后，`dist_dir` 中通常包含：

- `index.html`：首页
- `<category>/index.html`：分类页
- `<article>/index.html`：文章页
- `search.json`：搜索索引
- `static/`：样式与脚本资源

## 开发与测试

```bash
cargo test
```

```bash
cargo run -- --help
```

## License

如需开源发布，建议在仓库中补充 `LICENSE` 文件并在此处声明协议（例如 MIT）。
