---
name: project-version-update
description: 根据git提交历史和语义化版本控制规则更新项目版本号，修改相关配置文件，并生成Conventional Commits提交信息
---

# 项目版本更新技能

当需要更新项目版本时，执行以下步骤：

## 步骤0：检查工作区状态

1. 运行 `git status` 查看当前工作区状态，确认是否有未提交的更改
2. 运行 `git diff` 或 `git diff --cached` 查看具体修改了哪些文件

## 步骤1：读取工作区修改内容

如果工作区有未提交的修改：
1. 使用 `git diff` 查看具体变更内容
2. 读取修改的文件，分析变更类型（feat/fix/docs/style/chore/refactor等）
3. 根据变更类型决定是否需要更新版本号

如果工作区干净：
1. 运行 `git log --oneline -10` 查看最近的提交历史
2. 运行 `git diff HEAD~1 --stat` 查看最近一次提交的变更内容

## 步骤2：分析变更类型

根据变更内容分析类型：
- 如果是工作区修改：根据文件内容判断变更类型
- 如果是最近提交：根据 Conventional Commits 规范分析：
  - `feat:` 开头的提交 → **MINOR** 版本升级（次版本）
  - `fix:` 开头的提交 → **PATCH** 版本升级（补丁版本）
  - `feat!:` 或 `fix!:` 或包含 BREAKING CHANGE 的提交 → **MAJOR** 版本升级（主版本）
  - 仅 `docs:`、`style:`、`chore:`、`refactor:` 等 → 可能不需要版本升级

## 步骤3：确定版本号更新

语义化版本控制规则：
- 主版本 (MAJOR).次版本 (MINOR).补丁版本 (PATCH)
- 读取当前版本号，根据变更类型决定如何升级

当前版本号分别位于：
- `src-tauri/Cargo.toml` 中的 `version`
- `src-tauri/tauri.conf.json` 中的 `version`
- `package.json` 中的 `version`

## 步骤4：更新配置文件

如需更新版本号，同步修改以下文件：

### 4.1 src-tauri/Cargo.toml
修改 `[package]` 下的 `version` 字段

### 4.2 src-tauri/tauri.conf.json
修改顶层的 `version` 字段

### 4.3 package.json
修改 `version` 字段

### 4.4 docs/index.html
即使 git 工作区干净（无待提交更改），也需要检查并更新此文件：
1. 第279行 badge 中的版本（如 `v0.5.0` → `v0.5.1`）
2. 第359行 Windows 下载链接
3. 第368行 macOS 下载链接
4. 第377行 Linux 下载链接
将下载链接中的旧版本号替换为新版本号

## 步骤5：生成提交信息

### 5.1 格式规范

```
<type>: <description>
```

- 使用 **英文** 编写
- 第一行不超过 60 个字符
- type 使用小写：feat, fix, docs, chore, refactor 等


### 5.2 常用 type

- `feat`: 新功能、版本升级
- `fix`: bug 修复
- `docs`: 文档更新
- `chore`: 其他杂项（配置、工具等）
- `refactor`: 重构

### 5.3 description 编写

使用祈使句，简洁描述：

**版本升级**：
- `bump v0.6.0, add dialog plugin and prompt update`
- `bump v0.5.1, fix CSS url() resolution for themes`

**功能/修复**：
- `add new translation engine`
- `fix memory leak in parser`
- `update version to v0.5.1 in index.html`

**杂项**：
- `nouse`
- `remove unused files`

### 5.4 示例

```
feat: bump v0.6.0, add dialog plugin and prompt update

fix: update version to v0.5.1 in index.html

docs: update version to v0.5.1 in index.html

chore: nouse

feat!: 重构 LLM 请求为 system/user 分离消息格式

BREAKING CHANGE: 消息格式变更为分离的 system/user 结构
```

## 步骤6：执行提交

1. 运行 `git add -A` 添加所有更改
2. 展示commit信息（使用引用块与其他内容区分）：
   > <type>: <description>
3. 使用 `question` 工具询问用户："是否确认提交？"，选项为：
   - 是：确认提交
   - 否：取消提交
4. 根据用户选择：
   - 如果用户选择"是"：运行 `git commit -m "提交信息"` 创建提交
   - 如果用户选择"否"：取消提交，结束任务
   - 如果用户选择"Type your own answer"：让用户提供修改意见，重新生成提交信息，重复步骤2-3
5. **不要运行 git push**

## 注意事项

- 工作区有未提交的修改时，先分析这些修改是否需要版本升级，再决定如何处理
- 如果版本号不需要更新，只需生成符合规范的提交信息即可，不需要修改文件
- 步骤6只对需要版本升级的更新执行，如果只是分析则不需要执行提交