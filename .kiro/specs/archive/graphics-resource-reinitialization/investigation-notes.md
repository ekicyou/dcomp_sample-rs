# Bevy ECS リソース削除検出の調査メモ

## 調査背景

GraphicsCore再初期化の実装において、リソース削除の検出方法が要件実現の鍵となる。

## 調査項目

### 1. Bevy ECS 0.17.2のリソース削除検出機能

#### 確認事項
- [ ] `RemovedComponents<T>`はResourceに対応しているか？
  - 現状の理解: Componentのみ対応、Resourceは非対応の可能性
  - 検証方法: 公式ドキュメントとソースコード確認

- [ ] `Commands::remove_resource::<T>()`実行後、システムから削除を検知できるか？
  - 検証方法: 単純なテストケース作成

- [ ] リソース削除後、`Option<Res<T>>`はどのタイミングで`None`になるか？
  - 同一フレーム内で即座に反映？
  - 次フレームで反映？
  - `apply_deferred()`後に反映？

#### 検証用テストコード案

```rust
#[test]
fn test_resource_removal_detection() {
    let mut world = World::new();
    
    // リソース登録
    world.insert_resource(GraphicsCore::new().unwrap());
    
    // 削除システム作成
    let mut remove_system = IntoSystem::into_system(
        |mut commands: Commands| {
            commands.remove_resource::<GraphicsCore>();
        }
    );
    
    // 検出システム作成
    let mut detect_system = IntoSystem::into_system(
        |graphics: Option<Res<GraphicsCore>>| {
            println!("GraphicsCore exists: {}", graphics.is_some());
        }
    );
    
    remove_system.initialize(&mut world);
    detect_system.initialize(&mut world);
    
    // パターン1: 即座に検出できるか？
    remove_system.run((), &mut world);
    detect_system.run((), &mut world);  // None? Some?
    
    // パターン2: apply_deferred後に検出できるか？
    remove_system.apply_deferred(&mut world);
    detect_system.run((), &mut world);  // None? Some?
}
```

### 2. RemovedComponentsの動作確認（比較対象）

#### 既存実装の確認
- `RemovedComponents<ChildOf>`が`tree_system.rs`で使用されている
- Componentの削除は`on_remove`フックでも検知可能

#### 検証項目
- [ ] `RemovedComponents<T>`の仕組みを理解
  - イベントキューに削除が記録される？
  - `Query`で自動的に検出される？

### 3. 代替アプローチの検討

#### Option A: ポーリング方式（gap-analysis.mdで提案）

```rust
#[derive(Resource, Default)]
struct PrevGraphicsCoreState {
    existed: bool,
}

fn detect_graphics_core_removal(
    graphics: Option<Res<GraphicsCore>>,
    mut prev_state: ResMut<PrevGraphicsCoreState>,
    // ... 無効化処理用のQuery
) {
    let exists_now = graphics.is_some();
    
    if prev_state.existed && !exists_now {
        // 削除を検出！
        eprintln!("[Detect] GraphicsCore was removed");
        // 依存コンポーネントを無効化
    }
    
    prev_state.existed = exists_now;
}
```

**メリット**:
- 確実に動作（ECS標準機能のみ使用）
- 実装が単純

**デメリット**:
- 毎フレーム実行（軽微なオーバーヘッド）
- 1フレーム遅延（削除と検出が同一フレームで完結しない）

#### Option B: 明示的な通知パターン

```rust
#[derive(Resource)]
struct GraphicsCoreRemovalEvent {
    generation: u64,
}

// GraphicsCore削除時に明示的に呼ぶ
fn request_graphics_core_removal(mut commands: Commands) {
    commands.remove_resource::<GraphicsCore>();
    commands.insert_resource(GraphicsCoreRemovalEvent { generation: 1 });
}

// イベント検出システム
fn handle_graphics_core_removal(
    removal_event: Option<Res<GraphicsCoreRemovalEvent>>,
    // ...
) {
    if let Some(event) = removal_event {
        // 削除を検出し、依存コンポーネント無効化
    }
}
```

**メリット**:
- 即座に検出可能
- 世代番号で複数回の削除・再作成を追跡可能

**デメリット**:
- 削除時に明示的なイベント発行が必要（自動ではない）
- 呼び出し側の責任が増える

#### Option C: GraphicsCoreをOptionでラップ

```rust
#[derive(Resource)]
struct GraphicsCoreContainer {
    core: Option<GraphicsCore>,
    generation: u64,
}

// リソース自体は削除せず、内部をNoneにする
fn invalidate_graphics_core(mut container: ResMut<GraphicsCoreContainer>) {
    container.core = None;
    container.generation += 1;
}

// 通常のChanged検出が使える
fn detect_graphics_core_change(
    container: Res<GraphicsCoreContainer>,
) {
    if container.is_changed() && container.core.is_none() {
        // 無効化を検出！
    }
}
```

**メリット**:
- `Changed<Res<T>>`で検出可能（標準機能）
- 世代番号で追跡容易

**デメリット**:
- 全てのGraphicsCore使用箇所で`.core.as_ref()?`のような間接アクセス必要
- 既存コードの大幅変更

### 4. 推奨アプローチの再評価

現時点での推奨: **Option A (ポーリング方式) + Option B (明示的イベント) のハイブリッド**

#### 理由
1. **確実性**: ポーリング方式は必ず動作
2. **柔軟性**: イベントパターンで即座の検出も可能
3. **後方互換**: 既存コードへの影響最小

#### 実装方針
```rust
// 自動検出（フォールバック）
schedule.add_systems(PostLayout, detect_graphics_core_removal_polling);

// 明示的削除時のイベント（推奨パス）
fn explicit_remove_graphics_core(mut commands: Commands) {
    commands.remove_resource::<GraphicsCore>();
    commands.insert_resource(GraphicsCoreInvalidationEvent::default());
}

schedule.add_systems(PostLayout, handle_invalidation_event);
```

## 次のアクション

### 優先度: 高
1. [ ] 検証用テストコード実装（`tests/resource_removal_test.rs`）
2. [ ] Bevy ECS 0.17.2のドキュメント・ソース確認
3. [ ] 検証結果に基づく設計方針の確定

### 優先度: 中
4. [ ] 選択したアプローチのプロトタイプ実装
5. [ ] パフォーマンス影響の測定

## 参考情報

### Bevy ECS関連ドキュメント
- bevy_ecs 0.17.2: https://docs.rs/bevy_ecs/0.17.2/
- RemovedComponents: https://docs.rs/bevy_ecs/0.17.2/bevy_ecs/removal_detection/struct.RemovedComponents.html
- Resource: https://docs.rs/bevy_ecs/0.17.2/bevy_ecs/system/trait.Resource.html

### 既存コード参照
- `crates/wintf/src/ecs/tree_system.rs`: RemovedComponents使用例
- `crates/wintf/src/ecs/window.rs`: ComponentHook使用例
