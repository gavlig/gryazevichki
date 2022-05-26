## Unstable branch
### Currently broken due to migration to newer rapier and bevy_rapier so unless you want to compare old and new versions branch gryazevichki_rapier_v0.12.0-alpha.0 offers a better experience

![cargo_run](https://github.com/gavlig/gryazevichki/blob/master/assets/README/cargo_run.gif)

Hello! This is a prototype of vehicle simulator based on [Rapier](https://github.com/dimforge/rapier) and [Bevy](https://github.com/bevyengine/bevy) written on Rust. Inspired by [Godot-6DOF-Vehicle-Demo](https://github.com/Saitodepaula/Godot-6DOF-Vehicle-Demo).  

## Parameters

![wheel_size](https://github.com/gavlig/gryazevichki/blob/master/assets/README/wheel_size.gif)

From Parameters window these can be adjusted:
- Size of front and rear wheels 
- Every individual wheel's density and sizes
- Vehicle's body density and sizes  

## Motivation

It started as a learning project to check out the underlying tech. The goal was to get a somewhat working vehicle without any tweaks from game code over the results of simulation using just rigid bodies, joints and motors (just like in 6DOF-Vehicle-Demo).  
Unlike Bullet (physics engine used in said demo), Rapier doesnt support 6 degrees of freedom joints yet, but with two chained revolute joints: one for wheel rotation(x-axis) and one for steering(y-axis), a stable and functional wheel can be made! Making four of those and attaching them to a box scaled by z-axis makes a nice wagon. Motors are used for accelerating and steering.

## Controls

```
W / S: gas / reverse
A / D: steer left / steer right
Mouse look: camera orbiting around vehicle
Esc: Toggle Show/Hide mouse cursor
Ctrl + Space: Toggle flying camera (wasd + space + shift)
Ctrl + Esc: Close app
```
