# kitty_desktop

`kitty_desktop` 是对 `kitty` 终端内核的桌面封装层项目，以上游仓库为基础：

- 上游内核仓库：<https://github.com/wuhaodesq/kitty.git>
- 项目目标：不改写终端内核，专注桌面化能力（启动器、配置可视化、会话管理、集成入口）。

## 当前实现进度

已落地 **Phase 1 的第一步：`core-adapter` PoC**：

- `kitty` 可执行文件探测（PATH）
- 版本读取（`kitty --version`）
- 启动参数拼装（工作目录、title、config、session、shell、extra args）
- CLI 入口（支持 `version` / `launch --dry-run`）

## 与上游 kitty 的关系

- `kitty_desktop` **复用** `kitty` 的终端能力（渲染、协议、性能特性）。
- `kitty_desktop` **新增** 桌面应用层（GUI + 配置管理 + 运行编排）。
- 对上游仓库保持可追踪版本映射，避免桌面层与内核版本漂移。

## 项目范围

### In Scope（首期）

1. **桌面启动器**
   - 图形化启动 kitty
   - 预置启动模板（工作目录、shell、主题、会话）
2. **配置中心**
   - 可视化编辑常用配置（字体、配色、快捷键）
   - 生成/同步到 kitty 配置文件
3. **会话管理**
   - 保存与恢复工作会话（窗口/标签、启动命令）
4. **诊断与入口**
   - 一键打开日志目录、配置目录、上游文档

### Out of Scope（首期不做）

- 不重写 kitty 渲染或终端协议层
- 不替代 kitty 全量高级配置能力
- 不破坏用户可直接编辑原生 kitty 配置的方式

## 目录结构（当前）

```text
kitty_desktop/
├─ core_adapter/
│  ├─ __init__.py
│  ├─ kitty_adapter.py      # 探测、版本读取、启动命令拼装
│  └─ cli.py               # PoC CLI 入口
├─ tests/
│  └─ test_core_adapter.py
└─ README.md
```

## 快速开始（PoC）

### 1) 检查版本

```bash
python -m core_adapter.cli version
```

### 2) 预览启动命令（不真正启动）

```bash
python -m core_adapter.cli launch --directory . --title kitty_desktop --dry-run
```

### 3) 带额外参数预览

```bash
python -m core_adapter.cli launch --dry-run -- --single-instance
```

## 实现计划（基于上游 kitty）

### Phase 0：上游对齐与技术预研

- 拉取并固定上游基线版本（`wuhaodesq/kitty` commit/tag）
- 明确 kitty 可执行文件探测规则（PATH、安装位置、版本校验）
- 整理首期需要暴露的 kitty 配置项白名单

### Phase 1：MVP 可运行版本

- [x] 实现 `core-adapter` 基础 PoC
- [ ] 实现最小 `desktop-shell`（启动 / 设置 / 退出）
- [ ] 实现最小 `config-service`（配置表单落盘）
- [ ] 实现最小 `session-service`（保存与恢复会话）

### Phase 2：可用性增强

- 主题实时预览
- 快捷键冲突检测
- 启动失败诊断（路径、权限、参数错误）
- 增加常用模板（开发/运维/远程）

### Phase 3：发布与维护

- 多平台打包（Linux/macOS/Windows）
- 构建上游版本兼容矩阵
- 发布用户文档与开发者接入文档

## 下一步（立即执行）

1. 增加 `desktop-shell` 最小 GUI（先只接入 launch / dry-run）
2. 为配置项设计 schema，并建立与 kitty 配置文件的双向映射
3. 引入会话模型并支持 JSON 持久化
