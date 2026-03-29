#![allow(unused)]

use std::time::Duration;

use bevy::{camera::ScalingMode, color::palettes::tailwind::{RED_400, YELLOW_300}, input::mouse::MouseButtonInput, math::bounding::{Aabb2d, BoundingCircle, IntersectsVolume}, prelude::*, render::render_resource::AsBindGroup, shader::ShaderRef, sprite_render::{AlphaMode2d, Material2d, Material2dPlugin}, time::common_conditions::on_timer};

//游戏窗口尺寸
pub const CANVAS_SIZE: Vec2 = Vec2::new(480., 270.);
//角色放大倍数
pub const PLAYER_SIZE: f32 = 32.0;
const PIPE_SIZE: Vec2 = Vec2::new(32., CANVAS_SIZE.y);
const GAP_SIZE: f32 = 100.;
pub const PIPE_SPEED: f32 = 200.;

#[derive(Component)]
pub struct Pipe;

#[derive(Component)]
pub struct PipeTop;

#[derive(Component)]
pub struct PipeBotton;

#[derive(Component)]
pub struct PointsGate;

//角色相关

//角色
#[derive(Component)]
#[require(Gravity(700.), Velocity)]
pub struct Player;

//重力
#[derive(Component)]
pub struct Gravity(pub f32);

//速度
#[derive(Component, Default)]
pub struct Velocity(pub f32);

//结束游戏事件
#[derive(Event)]
pub struct EndGame;

//分数
#[derive(Resource, Default)]
pub struct Score(pub u32);

//触发加分事件
#[derive(Event)]
pub struct ScorePoint;

//结束文本标记
#[derive(Component)]
pub struct ScoreText;


pub struct PipPlugin;
impl Plugin for PipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (
            shift_pipes_to_the_left,
            spawn_pipes.run_if(on_timer(Duration::from_millis(1000))),
            despawn_pipes,
            check_collisions.after(spawn_pipes).after(shift_pipes_to_the_left),
            enforce_bird_direction
        ));
        app.add_observer(add_score);
    }
}
//上下管道和中间计算分的位置
fn spawn_pipes(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    time: Res<Time>
) {
    let image = asset_server.load("pipe.png");
    //按照九宫格分割
    let image_mode = SpriteImageMode::Sliced(
        TextureSlicer {
            border: BorderRect::axes(7., 10.),
            //中间那一块的渲染方式
            center_scale_mode: SliceScaleMode::Stretch,
            ..default()
        }
    );

    let transform = Transform::from_xyz(CANVAS_SIZE.x / 2., 0., 1.);
    let pipe_offset = PIPE_SIZE.y / 2. + GAP_SIZE / 2.;
    let gap_y_position = (time.elapsed_secs() * 4.2309875)
    .sin()
    * CANVAS_SIZE.y
    / 4.;

    commands.spawn((
        transform,
        Visibility::Visible,
        Pipe,
        children![(
            Sprite {
                image: image.clone(),
                custom_size: Some(PIPE_SIZE),
                image_mode: image_mode.clone(),
                ..default()
            },
            Transform::from_xyz(0., pipe_offset + gap_y_position, 1.),
            PipeTop
        ),
        (

            Visibility::Hidden,
            Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(10., GAP_SIZE)),
                ..default()
            },
            Transform::from_xyz(0., gap_y_position, 1.),
            PointsGate,
        ),
        (
            Sprite {
                image,
                custom_size: Some(PIPE_SIZE),
                image_mode,
                ..default()
            },
            Transform::from_xyz(0., -pipe_offset + gap_y_position, 1.),
            PipeBotton
        )]
    ));
}
//设置管道移动
pub fn shift_pipes_to_the_left(
    mut pipes: Query<&mut Transform, With<Pipe>>,
    time: Res<Time>    
) {
    for mut pipe in &mut pipes {
        //同步速度,防止低刷新率的情况下出现移动缓慢的情况
        pipe.translation.x -= PIPE_SPEED * time.delta_secs();
        //pipe.translation.x = pipe.translation.x.round();
    }
}

//处理出界管道
fn despawn_pipes(
    mut commands: Commands,
    pipes: Query<(Entity, &Transform), With<Pipe>>
) {
    //当管道到了屏幕边缘往左一个位置,就销毁
    for (entity, transform) in pipes.iter() {
        if transform.translation.x < -(CANVAS_SIZE.x / 2. + PIPE_SIZE.x) {
            commands.entity(entity).despawn();
        }
    }
}

//计算碰撞
fn check_collisions(
    mut commands: Commands,
    player: Single<(&Sprite, Entity), With<Player>>,
    pipe_segments: Query<
        (&Sprite, Entity),
        Or<(With<PipeTop>, With<PipeBotton>)>
    >,
    pipe_gaps: Query<(&Sprite, Entity), With<PointsGate>>,
    mut gizmos: Gizmos,
    //相对坐标转化为全局坐标
    transform_helper: TransformHelper
) -> Result<()> {
    let player_transform = transform_helper
        .compute_global_transform(player.1)?;
    //创建一个圆形碰撞箱
    let player_collider = BoundingCircle::new(
    player_transform.translation().xy(),
        PLAYER_SIZE / 2.
    );
    //标注碰撞箱
    gizmos.circle_2d(player_transform.translation().xy(), PLAYER_SIZE / 2., RED_400);

    for (sprite, entity) in &pipe_segments {
        //获取原始的坐标
        let pipe_transform = transform_helper.compute_global_transform(entity)?;
        //柱子碰撞箱
        let pipe_collider = Aabb2d::new(
            pipe_transform.translation().xy(),
            sprite.custom_size.unwrap() / 2.
        );
        //画一个框
        gizmos.rect_2d(pipe_transform.translation().xy(), sprite.custom_size.unwrap(), YELLOW_300);
        //碰到柱子结束
        if player_collider.intersects(&pipe_collider) {
            commands.trigger(EndGame);
        }
    }
    //穿过缝隙
    for (sprite, entity) in &pipe_gaps {
        //获取原始的坐标
        let pipe_transform = transform_helper.compute_global_transform(entity)?;
        //缝隙的碰撞箱
        let pipe_collider = Aabb2d::new(
            pipe_transform.translation().xy(),
            sprite.custom_size.unwrap() / 2.
        );
        //画一个框
        gizmos.rect_2d(pipe_transform.translation().xy(), sprite.custom_size.unwrap(), YELLOW_300);
        if player_collider.intersects((&pipe_collider)) {
            commands.trigger(ScorePoint);
            commands.entity(entity).despawn();
        }
    }
    Ok(())
}

//加分
fn add_score(_: On<ScorePoint>, mut score: ResMut<Score>) {
    score.0 += 1;
}

//更新分数
pub fn score_update(
    mut query: Query<&mut Text, With<ScoreText>>,
    score: Res<Score>
) {
    if score.is_changed() {
        for mut span in &mut query {
            span.0 = score.0.to_string();
        }
    }
} 


#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BackgroundMaterial {
    //图片
    #[texture(0)]
    //采样器,循环采样就是通过这个实现的
    #[sampler(1)]
    pub color_texture: Handle<Image>,
    
    #[uniform(2)]
    pub speed: f32
    
    
}

impl Material2d for BackgroundMaterial {
    //片元着色器,用来决定每一个像素是什么颜色
    fn fragment_shader() -> ShaderRef {
        //使用这个文件里的方式来渲染背景
        "background.wgsl".into()
    }
    //告诉gpu图片是透明的
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
/* //用来标记当前是否暂停
#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct Pause(pub bool);

#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PausableSystems;

fn pause(mut next_pause: ResMut<NextState<Pause>>) {
    next_pause.set(Pause(true));
} */

fn enforce_bird_direction(
    mut player: Single<
        (&mut Transform, &Velocity),
        With<Player>,
    >,
) {
    let calculated_velocity =
        Vec2::new(PIPE_SPEED, player.1.0);
    player.0.rotation = Quat::from_rotation_z(
        calculated_velocity.to_angle(),
    );
}