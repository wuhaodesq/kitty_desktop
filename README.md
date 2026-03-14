# kitty_desktop

`kitty_desktop` 是对 `kitty` 终端内核的桌面封装层项目，以上游仓库为基础：

- 上游内核仓库：<https://github.com/wuhaodesq/kitty.git>
- 项目目标：不改写终端内核，专注桌面化能力（启动器、配置可视化、会话管理、集成入口）。

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

## 建议目录结构（规划）

```text
kitty_desktop/
├─ desktop-shell/         # GUI 壳层（桌面应用入口）
├─ core-adapter/          # kitty 探测、版本检查、参数拼装
├─ config-service/        # GUI 配置 <-> kitty 配置映射
├─ session-service/       # 会话持久化与恢复
├─ packaging/             # 打包与发布脚本
└─ docs/                  # 设计与用户文档
```

## 实现计划（基于上游 kitty）

### Phase 0：上游对齐与技术预研

- 拉取并固定上游基线版本（`wuhaodesq/kitty` commit/tag）
- 明确 kitty 可执行文件探测规则（PATH、安装位置、版本校验）
- 整理首期需要暴露的 kitty 配置项白名单

### Phase 1：MVP 可运行版本

- 实现 `core-adapter`：
  - 检测 kitty 是否可用
  - 组装启动参数并拉起实例
- 实现最小 `desktop-shell`：
  - 启动 / 设置 / 退出
- 实现最小 `config-service`：
  - 配置表单落盘到 kitty 配置
- 实现最小 `session-service`：
  - 保存与恢复基础会话

### Phase 2：可用性增强

- 主题实时预览
- 快捷键冲突检测
- 启动失败诊断（路径、权限、参数错误）
- 增加常用模板（开发/运维/远程）

### Phase 3：发布与维护

- 多平台打包（Linux/macOS/Windows）
- 构建上游版本兼容矩阵
- 发布用户文档与开发者接入文档

## 里程碑交付物

- **M1（PoC）**：可探测并图形化启动 kitty
- **M2（MVP）**：可配置 + 可保存会话 + 可恢复会话
- **M3（Beta）**：具备诊断和模板能力，可进行小范围试用
- **M4（GA）**：完成多平台打包与版本兼容声明

## 下一步（立即执行）

1. 以 `https://github.com/wuhaodesq/kitty.git` 建立上游版本跟踪文档（tag/commit 对应关系）
2. 先实现 `core-adapter` 命令行 PoC，验证探测与参数注入
3. 再实现最小 GUI 壳层，形成“可启动 + 可配置 + 可恢复会话”的闭环
