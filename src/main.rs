use std::f32::consts::PI;

use bevy::math::Vec3Swizzles;
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use rusalka::NoiseGenerator;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(TankConfig {
            // 插入 TankConfig 资源
            tank_count: 20,        // 坦克数量
            safe_zone_radius: 8.0, // 安全区域半径
        })
        .init_resource::<CannonBallMesh>() // 初始化 CannonBallMesh 资源
        .add_startup_systems((setup, tank_spawn)) // 仅仅启动时调用一次
        .add_systems((
            // 每帧调用
            tank_move, // 坦克移动
            cannon_ball_velocity, // 根据炮弹速度与重力更新自身位置
            check_safe_zone, // 检测安全区域
            turret_rotate, // 坦克转台旋转
            turret_shoot.after(turret_rotate), // 坦克转台发射，在 turret_rotate 之后运行
        ))
        .run();
}

// 配置
#[derive(Resource)]
pub struct TankConfig {
    tank_count: u32,
    safe_zone_radius: f32,
}

// 坦克
#[derive(Component)]
pub struct Tank;

// 转台
#[derive(Component)]
pub struct Turret {
    spawn_point: Entity,
}

// 大炮
#[derive(Component)]
pub struct Cannon;

// 生成点
#[derive(Component)]
pub struct SpawnPoint;

// 炮弹
#[derive(Component)]
pub struct CannonBall {
    velocity: Vec3,
}

// 是否发射
#[derive(Component)]
pub struct Shooting;

// 炮弹的 Mesh
#[derive(Resource)]
pub struct CannonBallMesh(Handle<Mesh>);

impl FromWorld for CannonBallMesh {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        Self(
            meshes.add(
                shape::UVSphere {
                    radius: 0.1,
                    ..default()
                }
                .into(),
            ),
        )
    }
}

fn setup(
    mut commands: Commands,
    tank_config: Res<TankConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 地平面
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(500.0).into()),
        material: materials.add(Color::GRAY.into()),
        ..default()
    });

    // 方向光
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(0.98, 0.95, 0.82),
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(1.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // 3D 摄像机
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-50.0, 20.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // 安全区域
    commands.spawn((
        PbrBundle {
            mesh: meshes
                .add(
                    shape::UVSphere {
                        radius: tank_config.safe_zone_radius,
                        ..default()
                    }
                    .into(),
                )
                .into(),
            material: materials.add(StandardMaterial {
                base_color: Color::rgba(0.2, 0.8, 0.2, 0.4),
                unlit: true,                  // 关闭灯光
                alpha_mode: AlphaMode::Blend, // 开启透明度
                ..default()
            }),
            ..default()
        },
        NotShadowCaster,   // 不投射阴影
        NotShadowReceiver, // 不接收阴影
    ));
}

// 坦克生成
fn tank_spawn(
    tank_config: Res<TankConfig>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tank_mesh = meshes.add(shape::Cube::new(1.0).into());
    let turret_mesh = meshes.add(
        shape::UVSphere {
            radius: 0.5,
            ..default()
        }
        .into(),
    );
    let cannon_mesh = meshes.add(
        shape::Cylinder {
            radius: 0.5,
            height: 2.0,
            ..default()
        }
        .into(),
    );
    for _ in 0..tank_config.tank_count {
        let material = materials.add(StandardMaterial {
            base_color: Color::hsl(rand::random::<f32>() * 360.0, 1.0, 0.5),
            ..default()
        });
        let spawn_point = commands
            .spawn((
                SpawnPoint,
                GlobalTransform::default(),
                Transform::from_xyz(0.0, 1.0, 0.0),
            ))
            .id();
        let cannon = commands
            .spawn((
                Cannon,
                PbrBundle {
                    mesh: cannon_mesh.clone(),
                    material: material.clone(),
                    transform: Transform::from_xyz(0.0, 0.5, 0.0)
                        .with_scale(Vec3::new(0.2, 0.5, 0.2)),
                    ..default()
                },
            ))
            .add_child(spawn_point)
            .id();

        let turret = commands
            .spawn((
                Turret { spawn_point },
                PbrBundle {
                    mesh: turret_mesh.clone(),
                    material: material.clone(),
                    transform: Transform::from_xyz(0.0, 0.5, 0.0)
                        .with_rotation(Quat::from_rotation_x(45.0)),
                    ..default()
                },
            ))
            .add_child(cannon)
            .id();
        commands
            .spawn((
                Tank,
                PbrBundle {
                    mesh: tank_mesh.clone(),
                    material: material.clone(),
                    transform: Transform::from_xyz(0.0, 0.5, 0.0),
                    ..default()
                },
            ))
            .add_child(turret);
    }
}

// 坦克在地面随机移动与旋转
fn tank_move(mut tanks: Query<(Entity, &mut Transform), With<Tank>> /*查询 Tank 的 Entity 与 Transform 组件*/, time: Res<Time>) {
    let dt = time.delta_seconds();
    let generator = NoiseGenerator::new("Nose");
    for (entity, mut transform) in tanks.iter_mut() {
        let mut pos = transform.translation;
        pos.y = entity.index() as f32;
        pos /= 10.0;
        // 设置随机的角度与位置
        let angle: f32 = (0.5 + generator.get(pos.x, pos.y, pos.z)) * 4.0 * PI;
        let (x, z) = angle.sin_cos();
        transform.rotation = Quat::from_rotation_y(angle);
        transform.translation += Vec3::new(x, 0.0, z) * dt * 5.0;
    }
}

// 坦克转台旋转
fn turret_rotate(mut turret: Query<&mut Transform, With<Turret>>, time: Res<Time>) {
    // 每秒旋转 180 度
    let rotation_y = Quat::from_rotation_y(time.delta_seconds() * PI);

    for mut transform in turret.iter_mut() {
        transform.rotation = rotation_y * transform.rotation;
    }
}

// 坦克转台发射
fn turret_shoot(
    mut commands: Commands,
    cannon_ball_mesh: Res<CannonBallMesh>,
    turrets: Query<(&Turret, &Handle<StandardMaterial>, &GlobalTransform), With<Shooting>>,// 查询包含Shooting组件的实体的 Turret、材质、全局变换数据
    global_transform_query: Query<&GlobalTransform>,
) {
    for (turret, material, global_transform) in turrets.iter() {
        let spawn_point_pos = global_transform_query
            .get(turret.spawn_point)
            .unwrap()
            .translation();
        commands.spawn((
            CannonBall {
                velocity: global_transform.up() * 20.0,
            },
            PbrBundle {
                material: material.clone(),
                transform: Transform::from_translation(spawn_point_pos),
                mesh: cannon_ball_mesh.0.clone(),
                ..default()
            },
        ));
    }
}
// 重力
const GRAVITY: Vec3 = Vec3::new(0.0, -9.82, 0.0);

const INVERT_Y: Vec3 = Vec3::new(1.0, -1.0, 1.0);

// 根据炮弹速度与重力更新自身位置
fn cannon_ball_velocity(
    mut cannon_balls: Query<(&mut CannonBall, &mut Transform, Entity)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let dt = time.delta_seconds();

    for (mut cannon_ball, mut transform, entity) in cannon_balls.iter_mut() {
        // 根据速度更改位置
        transform.translation += cannon_ball.velocity * dt;

        // 下降到地面时反弹，速度下降至 0.8
        if transform.translation.y < 0.0 {
            transform.translation *= INVERT_Y;
            cannon_ball.velocity *= INVERT_Y * 0.8;
        }

        // 重力加速度影响炮弹速度
        cannon_ball.velocity += GRAVITY * dt;

        // 炮弹速度小于 0.1 时 摧毁
        if cannon_ball.velocity.length_squared() < 0.1 {
            commands.entity(entity).despawn();
        }
    }
}

// 检测安全区域
fn check_safe_zone(
    turrets: Query<(Entity, &GlobalTransform, Option<&Shooting>), With<Turret>>, // 查询 Turret 的 Entity、全局转换、可选的 Shooting 组件数据
    tank_config: Res<TankConfig>,
    mut commands: Commands,
) {
    for (entity, global_transform, shooting) in turrets.iter() {
        // 判断是否离开了 安全区域，如果是则 插入 Shooting 组件，标识它 去发射炮弹
        if global_transform.translation().xz().length() > tank_config.safe_zone_radius {
            if shooting.is_none() {
                commands.entity(entity).insert(Shooting);
            }
        } else {
            if shooting.is_some() {
                commands.entity(entity).remove::<Shooting>();
            }
        }
    }
}