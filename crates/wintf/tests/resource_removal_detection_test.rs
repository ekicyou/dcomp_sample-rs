//! Bevy ECS Resource削除検出の検証テスト
//!
//! このテストは以下を検証します:
//! - Commands::remove_resource実行後、いつResourceが削除されるか
//! - Option<Res<T>>でResourceの不在を検出できるか
//! - apply_deferred()のタイミングによる挙動の違い

use bevy_ecs::prelude::*;

/// テスト用のダミーResource
#[derive(Resource, Debug, Clone, PartialEq)]
struct DummyResource {
    value: i32,
}

/// 前フレームの状態を記録するResource
#[derive(Resource, Debug, Default)]
struct PreviousResourceState {
    existed: bool,
    removal_detected: bool,
}

#[test]
fn test_resource_removal_immediate_detection() {
    // Commands::remove_resource実行直後に削除を検出できるか

    let mut world = World::new();
    world.insert_resource(DummyResource { value: 42 });

    // 削除システム
    let mut remove_system = IntoSystem::into_system(|mut commands: Commands| {
        eprintln!("[Remove System] リソースを削除します");
        commands.remove_resource::<DummyResource>();
    });

    // 検出システム
    let mut detect_system = IntoSystem::into_system(|res: Option<Res<DummyResource>>| {
        let exists = res.is_some();
        if let Some(r) = res {
            eprintln!("[Detect System] リソースが存在: value={}", r.value);
        } else {
            eprintln!("[Detect System] リソースが存在しない");
        }
        exists
    });

    remove_system.initialize(&mut world);
    detect_system.initialize(&mut world);

    // 初期状態: リソースが存在
    let exists_before = detect_system.run((), &mut world).expect("システム実行成功");
    assert!(exists_before, "削除前はリソースが存在すること");

    // リソース削除実行
    remove_system.run((), &mut world).expect("システム実行成功");

    // apply_deferred前に検出（即座に反映されるか？）
    let exists_after_remove = detect_system.run((), &mut world).expect("システム実行成功");
    eprintln!(
        "[Test] apply_deferred前: リソース存在={} (期待: true - まだ削除されていない)",
        exists_after_remove
    );

    // apply_deferred実行
    remove_system.apply_deferred(&mut world);

    // apply_deferred後に検出
    let exists_after_deferred = detect_system.run((), &mut world).expect("システム実行成功");
    eprintln!(
        "[Test] apply_deferred後: リソース存在={} (期待: false - 削除完了)",
        exists_after_deferred
    );

    assert!(
        !exists_after_deferred,
        "apply_deferred後はリソースが削除されていること"
    );
}

#[test]
fn test_resource_removal_polling_pattern() {
    // ポーリング方式で削除を検出できるか

    let mut world = World::new();
    world.insert_resource(DummyResource { value: 100 });
    world.insert_resource(PreviousResourceState::default());

    // ポーリング検出システム
    let mut polling_system = IntoSystem::into_system(
        |res: Option<Res<DummyResource>>, mut prev: ResMut<PreviousResourceState>| {
            let exists_now = res.is_some();

            if prev.existed && !exists_now {
                eprintln!("[Polling] リソース削除を検出！");
                prev.removal_detected = true;
            }

            prev.existed = exists_now;
        },
    );

    polling_system.initialize(&mut world);

    // 1回目: 初期状態記録
    polling_system
        .run((), &mut world)
        .expect("システム実行成功");
    polling_system.apply_deferred(&mut world);

    {
        let prev = world.resource::<PreviousResourceState>();
        assert!(prev.existed, "初回実行でリソース存在を記録");
        assert!(!prev.removal_detected, "まだ削除は検出されていない");
    }

    // リソースを削除
    world.remove_resource::<DummyResource>();

    // 2回目: 削除を検出
    polling_system
        .run((), &mut world)
        .expect("システム実行成功");
    polling_system.apply_deferred(&mut world);

    {
        let prev = world.resource::<PreviousResourceState>();
        assert!(!prev.existed, "削除後は存在フラグがfalse");
        assert!(prev.removal_detected, "ポーリングで削除を検出できること");
    }

    eprintln!("✅ ポーリング方式での削除検出に成功");
}

#[test]
fn test_resource_removal_in_schedule() {
    // スケジュール内での削除と検出の動作確認

    let mut world = World::new();
    world.insert_resource(DummyResource { value: 200 });
    world.insert_resource(PreviousResourceState::default());

    let mut schedule = Schedule::default();

    // 削除システムをスケジュールに追加
    schedule.add_systems(
        (|mut commands: Commands| {
            eprintln!("[Schedule] リソースを削除");
            commands.remove_resource::<DummyResource>();
        })
        .run_if(|res: Option<Res<DummyResource>>| res.is_some()),
    );

    // ポーリング検出システムを追加
    schedule.add_systems(
        |res: Option<Res<DummyResource>>, mut prev: ResMut<PreviousResourceState>| {
            let exists_now = res.is_some();
            eprintln!("[Schedule Polling] リソース存在: {}", exists_now);

            if prev.existed && !exists_now {
                eprintln!("[Schedule Polling] 削除検出！");
                prev.removal_detected = true;
            }

            prev.existed = exists_now;
        },
    );

    // 1回目: リソース存在、削除実行
    eprintln!("\n=== 1回目のスケジュール実行 ===");
    schedule.run(&mut world);

    {
        let prev = world.resource::<PreviousResourceState>();
        eprintln!(
            "[Test] 1回目実行後 - existed: {}, removal_detected: {}",
            prev.existed, prev.removal_detected
        );
    }

    // 2回目: 削除が反映され、検出される
    eprintln!("\n=== 2回目のスケジュール実行 ===");
    schedule.run(&mut world);

    {
        let prev = world.resource::<PreviousResourceState>();
        eprintln!(
            "[Test] 2回目実行後 - existed: {}, removal_detected: {}",
            prev.existed, prev.removal_detected
        );

        assert!(
            prev.removal_detected,
            "2回目のスケジュール実行で削除が検出されること（1フレーム遅延）"
        );
    }

    eprintln!("✅ スケジュール内でのポーリング削除検出に成功（1フレーム遅延）");
}

#[test]
fn test_explicit_removal_event_pattern() {
    // 明示的なイベントResourceを使った削除通知パターン

    #[derive(Resource, Default)]
    struct RemovalEvent {
        triggered: bool,
    }

    let mut world = World::new();
    world.insert_resource(DummyResource { value: 300 });

    // 削除+イベント発行システム
    let mut remove_with_event = IntoSystem::into_system(|mut commands: Commands| {
        eprintln!("[Explicit] リソース削除とイベント発行");
        commands.remove_resource::<DummyResource>();
        commands.insert_resource(RemovalEvent { triggered: true });
    });

    // イベント検出システム
    let mut detect_event = IntoSystem::into_system(
        |event: Option<Res<RemovalEvent>>, res: Option<Res<DummyResource>>| {
            if let Some(e) = event {
                if e.triggered {
                    eprintln!(
                        "[Explicit Detect] 削除イベント検出, リソース存在: {}",
                        res.is_some()
                    );
                    return true;
                }
            }
            false
        },
    );

    remove_with_event.initialize(&mut world);
    detect_event.initialize(&mut world);

    // 削除実行
    remove_with_event
        .run((), &mut world)
        .expect("システム実行成功");
    remove_with_event.apply_deferred(&mut world);

    // イベント検出（即座に検出可能）
    let event_detected = detect_event.run((), &mut world).expect("システム実行成功");
    assert!(
        event_detected,
        "明示的イベントパターンで即座に削除を検出できること"
    );

    eprintln!("✅ 明示的イベントパターンでの削除通知に成功");
}

#[test]
fn test_optional_wrapper_pattern() {
    // ResourceをOptionでラップするパターン

    #[derive(Resource, Debug)]
    struct ResourceContainer {
        inner: Option<DummyResource>,
        generation: u64,
    }

    let mut world = World::new();
    world.insert_resource(ResourceContainer {
        inner: Some(DummyResource { value: 400 }),
        generation: 0,
    });

    // 無効化システム（削除ではなく内部をNoneにする）
    let mut invalidate_system =
        IntoSystem::into_system(|mut container: ResMut<ResourceContainer>| {
            eprintln!("[Optional] リソースを無効化（内部をNoneに）");
            container.inner = None;
            container.generation += 1;
        });

    // Changed検出システム
    let mut detect_change = IntoSystem::into_system(|container: Res<ResourceContainer>| {
        if container.is_changed() {
            eprintln!(
                "[Optional Detect] 変更検出, 有効: {}, 世代: {}",
                container.inner.is_some(),
                container.generation
            );
            return true;
        }
        false
    });

    invalidate_system.initialize(&mut world);
    detect_change.initialize(&mut world);

    // 初回検出（初期化直後なのでis_changed()がtrue）
    let changed_initial = detect_change.run((), &mut world).expect("システム実行成功");
    eprintln!("[Test] 初回: changed={}", changed_initial);

    // 無効化実行
    invalidate_system
        .run((), &mut world)
        .expect("システム実行成功");
    invalidate_system.apply_deferred(&mut world);

    // 変更検出
    let changed_after = detect_change.run((), &mut world).expect("システム実行成功");
    assert!(changed_after, "Optionalラップパターンで変更検出できること");

    // リソース確認
    {
        let container = world.resource::<ResourceContainer>();
        assert!(
            container.inner.is_none(),
            "内部リソースが無効化されていること"
        );
        assert_eq!(container.generation, 1, "世代番号が更新されていること");
    }

    eprintln!("✅ Optionalラップパターンでの変更検出に成功");
}

#[test]
fn test_resource_removal_detection_summary() {
    eprintln!("\n========================================");
    eprintln!("  Bevy ECS Resource削除検出まとめ");
    eprintln!("========================================\n");

    eprintln!("✅ 検証完了した検出パターン:");
    eprintln!("  1. ポーリング方式: 毎フレーム存在チェック（1フレーム遅延）");
    eprintln!("  2. 明示的イベント: 削除時に専用Resourceで通知（即座）");
    eprintln!("  3. Optionalラップ: Changed検出が使用可能（即座）");
    eprintln!();
    eprintln!("⚠️  重要な知見:");
    eprintln!("  - Commands::remove_resource()はapply_deferred()後に反映");
    eprintln!("  - RemovedComponents<T>はResourceに対応していない");
    eprintln!("  - ポーリング方式は確実だが1フレーム遅延がある");
    eprintln!();
    eprintln!("推奨: ポーリング方式 + 明示的イベントのハイブリッド");
    eprintln!("========================================\n");
}
