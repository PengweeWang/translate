---
name: auto-commit-generator
description: 基于git diff和Conventional Commits规范，自动分析变更并生成规范的提交信息
---

# 自动生成Commit信息

当需要提交代码变更时，执行以下步骤生成符合规范的commit信息。

## 步骤1：检查工作区状态

1. 运行 `git status` 查看当前工作区状态，确认是否有未提交的更改
2. 运行 `git diff` 查看具体修改了哪些内容（包含 unstaged changes）
3. 运行 `git diff --cached` 查看已暂存的内容（staged changes）


## 步骤2：分析变更内容

根据diff内容分析变更类型：
- **feat**: 新增功能、特性
- **fix**: bug修复
- **docs**: 文档更新
- **style**: 代码格式调整（不影响功能）
- **refactor**: 代码重构
- **perf**: 性能优化
- **test**: 测试相关
- **chore**: 构建过程或辅助工具变动
- **ci**: CI配置文件或脚本更新
- **build**: 构建系统或外部依赖变更

## 步骤3：生成提交信息

### 格式规范

根据 [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/) 规范：

```
<type>[optional scope][!]: <description>

[optional body]

[optional footer(s)]
```

**说明：**

1. **type（必需）**：提交类型
   - `feat`: 新功能
   - `fix`: bug修复
   - `docs`: 文档
   - `style`: 格式调整
   - `refactor`: 重构
   - `perf`: 性能优化
   - `test`: 测试
   - `chore`: 构建/工具
   - `ci`: CI配置
   - `build`: 构建系统

2. **scope（可选）**：范围，如 `feat(parser):`

3. **`!`（可选）**：标记breaking change，如 `feat(api)!:`

4. **description（必需）**：简短描述，不超过50字符，动词开头

5. **body（可选）**：详细说明

6. **footer（可选）**：
   - `BREAKING CHANGE: <description>`
   - `Refs: #123`

**示例：**

```
feat: add new translation feature

fix: prevent racing of requests

BREAKING CHANGE: environment variables now take precedence
```

## 步骤4：执行提交

1. 运行 `git add -A` 添加所有更改

2. 展示commit信息（使用引用块）：
   > \<type\>[optional scope][!]: <description>
   >
   > [optional body]
   >
   > [optional footer(s)]

3. 使用 `question` 工具询问用户："是否确认提交？"，选项为：
   - 是：确认提交
   - 否：取消提交

4. 根据用户选择：
   - 选择"是"：运行 `git commit -m "提交信息"` 创建提交
   - 选择"否"：取消提交
   - 其他输入：重新生成提交信息

5. **不要运行 git push**

## 注意事项

- 严格按照Conventional Commits格式生成提交信息
- 描述应简洁明了，用动词开头（add, fix, update, remove等）
- 未经用户确认，不得执行提交