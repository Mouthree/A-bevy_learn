use bevy::color::palettes::tailwind::SLATE_50;
use bevy::image::{ImageAddressMode, ImageLoaderSettings};
use bevy::sprite_render::Material2dPlugin;
use bevy_work::*;
use bevy::app::AppExit;
use bevy::camera::ScalingMode;
use bevy::prelude::*;



fn main() -> AppExit {
    App::new()
        .init_resource::<Score>()
        //.init_state::<Pause>()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((
                PipPlugin,
                //根据AsBindGroup里面写的来将它转为GPU可以识别的资源绑定
                Material2dPlugin::<BackgroundMaterial>::default()
            ))
        .add_systems(Startup, startup)
        .add_systems(FixedUpdate, (gravity, check_in_bounds))
        .add_systems(Update, (
            controls,
            //数据更新了才调用这个函数
            score_update.run_if(resource_changed::<Score>)
        ))
        .add_observer(respawn_on_endgame)
        //.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))))
        .run()
}

//初始化
fn startup(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    mut config_store: ResMut<GizmoConfigStore>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BackgroundMaterial>>
) {
    //commands.spawn(Camera2d);
    commands.spawn((
        Player,
        //添加一个精灵
        Sprite {
            //覆盖原本纹理,设置显示大小
            custom_size: Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
            image: asset_server.load("lu.png"),
            //其余属性都用默认值
            ..default()
        },
        //设置坐标位置,2d中z轴用来调整显示层级
        Transform::from_xyz(-CANVAS_SIZE.x / 4., 0.0, 1.0),
    ));
    commands.spawn((
        Camera2d,
        //定义相机投影为正交投影
        Projection::Orthographic(OrthographicProjection {
            //设定视角自动缩放且有最大值,保持比例为下面两个值
            scaling_mode: ScalingMode::AutoMax {
                max_width: CANVAS_SIZE.x,
                max_height: CANVAS_SIZE.y,
            },
            //剩下的配置项都使用默认的2D值
            ..OrthographicProjection::default_2d()
        }),
    ));

    //开关碰撞箱的框
    let (config, _) = config_store
        .config_mut::<DefaultGizmoConfigGroup>();
    config.enabled = false;

    //记分的文字
    commands.spawn((
        //这玩意调整的是整个框的位置大小等
        Node {
            //百分比的
            width: percent(100.),
            margin: px(20.).top(),
            ..default()
        },
        Text::new("0"),
        //水平居中
        TextLayout::new_with_justify(Justify::Center),
        TextFont {
            font_size: 33.,
            ..default()
        },
        TextColor(SLATE_50.into()),
        ScoreText
    ));

    //背景1
    commands.spawn((
        //定义实体形状
        Mesh2d(meshes.add(Rectangle::new(
            CANVAS_SIZE.x,
            CANVAS_SIZE.y
        ))),
        //定义外观,贴图等
        MeshMaterial2d(materials.add(BackgroundMaterial {
            color_texture: asset_server.load_with_settings(
                "background1.png", 
                |settings: &mut ImageLoaderSettings| {
                    settings
                        //获取图片的采样器设置
                        .sampler
                        //获取采样器描述符
                        .get_or_init_descriptor()
                        //设置图片模式是循环的
                        .set_address_mode(
                            ImageAddressMode::Repeat
                        );
                }
            ),  
            speed: 0.2,
        })),
        Transform::from_xyz(0., 0., -2.)
    ));
    //背景2
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(
            CANVAS_SIZE.x,
            CANVAS_SIZE.y
        ))),
        MeshMaterial2d(materials.add(BackgroundMaterial {
            color_texture: asset_server.load_with_settings(
                "background2.png", 
                |settings: &mut ImageLoaderSettings| {
                    settings
                    .sampler
                    .get_or_init_descriptor()
                    .set_address_mode(ImageAddressMode::Repeat);
                }
            ),
            speed: 0.05,
        })),
        Transform::from_xyz(0., 0., -1.)
    ));

}

//重力
fn gravity(
    //遍历所有有这三个组件的实体
    mut transforms: Query<(
        &mut Transform,
        &mut Velocity,
        &Gravity
    )>,
    time: Res<Time>
) {
    for(mut transform, mut velocity, gravity) in &mut transforms {
        velocity.0 -= gravity.0 * time.delta_secs();
        transform.translation.y += velocity.0 * time.delta_secs();
    }
}

//按键控制
fn controls(
    velocity: Single<&mut Velocity, With<Player>>,
    buttons: Res<ButtonInput<MouseButton>>
) {
    let mut velocity = velocity.into_inner();
    if buttons.any_just_pressed([
        MouseButton::Left,
        MouseButton::Right
    ]) {
        velocity.0 = 200.;
    }
}

//边界检测
fn check_in_bounds(player: Single<&Transform, With<Player>>, mut commands: Commands) {
    if player.translation.y < -CANVAS_SIZE.y / 2.0 - PLAYER_SIZE || 
    player.translation.y > CANVAS_SIZE.y / 2.0 + PLAYER_SIZE {
        commands.trigger(EndGame);
    }
}

//触发边界检测后的处理
fn respawn_on_endgame(
    _: On<EndGame>, 
    //mut commands: Commands, 
    player: Single<(&mut Transform, &mut Velocity), With<Player>>,
    mut score: ResMut<Score>
) {
    //需要将player改为Entry⬇️,这种方式是直接替换组件,感觉不如修改值
    /* commands.entity(*player).insert((
        Transform::from_xyz(-CANVAS_SIZE.x / 4., 0., 1.),
        Velocity(0.)
    )); */

    let (mut transform, mut velocity) = player.into_inner();
    transform.translation = Vec3 { x: -CANVAS_SIZE.x / 4., y: 0., z: 1. };
    velocity.0 = 0.;
    score.0 = 0;
}