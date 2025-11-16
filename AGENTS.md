# AI-DLC and Spec-Driven Development

Kiro-style Spec Driven Development implementation on AI-DLC (AI Development Life Cycle)

## Project Context

### Paths
- Steering: `.kiro/steering/`
- Specs: `.kiro/specs/`

### Steering vs Specification

**Steering** (`.kiro/steering/`) - Guide AI with project-wide rules and context
**Specs** (`.kiro/specs/`) - Formalize development process for individual features

### Active Specifications
- Check `.kiro/specs/` for active specifications
- Use `/kiro-spec-status [feature-name]` to check progress

### Completed Specifications Archive
- Completed specs should be moved to `.kiro/specs/archive/` to keep active specs focused
- Archive criteria:
  - All implementation tasks are completed
  - Code is merged to main branch
  - No pending issues or follow-up work
- Use `/kiro-spec-status {feature}` to verify completion before archiving
- Archive command: `Move-Item .kiro/specs/{feature} .kiro/specs/archive/{feature}`

## Development Guidelines
- Think in English, but generate responses in Japanese (思考は英語、回答の生成は日本語で行うように)

## Coding Rules
- テストコードは`tests/`ディレクトリに配置し、ソースコードの肥大化を避ける

## Git Commit Rules
- Gitコミットメッセージは日本語で記述すること
- フォーマット: `<type>: <subject>` (例: `feat: 新機能追加`, `fix: バグ修正`, `docs: ドキュメント更新`)

## Minimal Workflow
- Phase 0 (optional): `/kiro-steering`, `/kiro-steering-custom`
- Phase 1 (Specification):
  - `/kiro-spec-init "description"`
  - `/kiro-spec-requirements {feature}`
  - `/kiro-validate-gap {feature}` (optional: for existing codebase)
  - `/kiro-spec-design {feature} [-y]`
  - `/kiro-validate-design {feature}` (optional: design review)
  - `/kiro-spec-tasks {feature} [-y]`
- Phase 2 (Implementation): `/kiro-spec-impl {feature} [tasks]`
  - `/kiro-validate-impl {feature}` (optional: after implementation)
- Progress check: `/kiro-spec-status {feature}` (use anytime)

## Development Rules
- 3-phase approval workflow: Requirements → Design → Tasks → Implementation
- Human review required each phase; use `-y` only for intentional fast-track
- Keep steering current and verify alignment with `/kiro-spec-status`

## Steering Configuration
- Load entire `.kiro/steering/` as project memory
- Default files: `product.md`, `tech.md`, `structure.md`
- Custom files are supported (managed via `/kiro-steering-custom`)
