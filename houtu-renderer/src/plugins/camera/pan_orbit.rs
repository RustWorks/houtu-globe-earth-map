use super::camera_event_aggregator::ControlEvent;
use super::camera_new::GlobeCamera;
use super::{egui, GlobeCameraControl};
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::math::{DMat4, DVec3};
use bevy::prelude::*;
use bevy::render::camera::{CameraProjection, RenderTarget};
use bevy::window::{PrimaryWindow, WindowRef};
use bevy_easings::Lerp;
use bevy_egui::{EguiSet, WindowSize};
use egui::EguiWantsFocus;
use houtu_scene::{
    acos_clamped, Cartesian3, Cartographic, Ellipsoid, HeadingPitchRoll, SceneTransforms,
};
use std::f64::consts::{PI, TAU};
pub fn pan_orbit_camera(
    mut mouse_motion: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    primary_query: Query<&Window, With<PrimaryWindow>>,
    mut orbit_cameras: Query<(
        Entity,
        &mut GlobeCamera,
        &mut Transform,
        &mut Projection,
        &mut GlobalTransform,
        &mut GlobeCamera,
        &GlobeCameraControl,
    )>,
    mut control_event_rader: EventReader<ControlEvent>,
) {
    let Ok(primary) = primary_query.get_single() else {
        return;
    };
    let window_size = Vec2 {
        x: primary.width(),
        y: primary.height(),
    };

    let mouse_delta = mouse_motion.iter().map(|event| event.delta).sum::<Vec2>();
    for event in control_event_rader.iter() {
        for (
            entity,
            mut pan_orbit,
            mut transform,
            mut projection,
            global_transform,
            globe_camera,
            globe_camera_control,
        ) in orbit_cameras.iter_mut()
        {
            let projection = if let Projection::Perspective(v) = *projection {
                v
            } else {
                return;
            };
            let camera_position_cartographic =
                if let Some(v) = globe_camera_control.position_cartographic {
                    v
                } else {
                    return;
                };
            match event {
                ControlEvent::Zoom(data) => {
                    let movement = data.movement;
                    let mut windowPosition;
                    if globe_camera_control._cameraUnderground {
                        windowPosition = movement.startPosition.clone();
                    } else {
                        windowPosition = Vec2::ZERO;
                        windowPosition.x = window_size.x / 2.0;
                        windowPosition.y = window_size.y / 2.0;
                    }
                    let ray = globe_camera.getPickRay(&windowPosition, &window_size, &projection);
                    let ray = if let Some(v) = ray {
                        v
                    } else {
                        continue;
                    };
                    let position = ray.origin;
                    let direction = ray.direction;
                    let height = camera_position_cartographic.height;
                    let normal = DVec3::UNIT_X;
                    let distance = height;
                    let unitPosition = globe_camera_control.position_cartesian.normalize();
                    let unitPositionDotDirection = unitPosition.dot(globe_camera_control.direction);
                    let mut percentage = 1.0;
                    percentage = unitPositionDotDirection.abs().clamp(0.25, 1.0);
                    let diff = (movement.endPosition.y - movement.startPosition.y) as f64;
                    // distanceMeasure should be the height above the ellipsoid.
                    // When approaching the surface, the zoomRate slows and stops minimumZoomDistance above it.
                    let distanceMeasure = distance;
                    let zoomFactor = globe_camera_control._zoomFactor;
                    let approachingSurface = diff > 0.;
                    let minHeight = {
                        if approachingSurface {
                            globe_camera_control.minimumZoomDistance * percentage
                        } else {
                            0.
                        }
                    };
                    let maxHeight = globe_camera_control.maximumZoomDistance;

                    let minDistance = distanceMeasure - minHeight;
                    let zoomRate = zoomFactor * minDistance;
                    zoomRate = zoomRate.clamp(
                        globe_camera_control._minimumZoomRate,
                        globe_camera_control._maximumZoomRate,
                    );
                    let startPosition = movement.startPosition;
                    let rangeWindowRatio = diff / window_size.y as f64;
                    rangeWindowRatio =
                        rangeWindowRatio.min(globe_camera_control.maximumMovementRatio);
                    let distance = zoomRate * rangeWindowRatio;

                    if globe_camera_control.enableCollisionDetection
                        || globe_camera_control.minimumZoomDistance == 0.0
                    // || !defined(globe_camera_control._globe)
                    // look-at mode
                    {
                        if (distance > 0.0 && (distanceMeasure - minHeight).abs() < 1.0) {
                            continue;
                        }

                        if (distance < 0.0 && (distanceMeasure - maxHeight).abs() < 1.0) {
                            continue;
                        }

                        if (distanceMeasure - distance < minHeight) {
                            distance = distanceMeasure - minHeight - 1.0;
                        } else if (distanceMeasure - distance > maxHeight) {
                            distance = distanceMeasure - maxHeight;
                        }
                    }

                    // let scene = globe_camera_control._scene;
                    // let camera = scene.camera;
                    // let mode = scene.mode;

                    let orientation = HeadingPitchRoll::default();
                    orientation.heading = globe_camera.hpr.heading;
                    orientation.pitch = globe_camera.hpr.pitch;
                    orientation.roll = globe_camera.hpr.roll;

                    let sameStartPosition = startPosition.eq(&globe_camera_control._zoomMouseStart);
                    let zoomingOnVector = globe_camera_control._zoomingOnVector;
                    let rotatingZoom = globe_camera_control._rotatingZoom;
                    let pickedPosition;

                    if (!sameStartPosition) {
                        pickedPosition =
                            globe_camera.pickEllipsoid(&startPosition, &window_size, &projection);

                        globe_camera_control._zoomMouseStart = startPosition.clone();
                        if (pickedPosition.is_some()) {
                            globe_camera_control._useZoomWorldPosition = true;
                            globe_camera_control._zoomWorldPosition =
                                pickedPosition.unwrap().clone();
                        } else {
                            globe_camera_control._useZoomWorldPosition = false;
                        }

                        zoomingOnVector = false;
                        globe_camera_control._zoomingOnVector = false;
                        rotatingZoom = false;
                        globe_camera_control._rotatingZoom = false;
                        globe_camera_control._zoomingUnderground =
                            globe_camera_control._cameraUnderground;
                    }

                    if (!globe_camera_control._useZoomWorldPosition) {
                        globe_camera.zoom_in(Some(distance));
                        return;
                    }

                    let zoomOnVector = false;

                    if (camera_position_cartographic.height < 2000000.) {
                        rotatingZoom = true;
                    }

                    if (!sameStartPosition || rotatingZoom) {
                        let cameraPositionNormal =
                            globe_camera_control.position_cartesian.normalize();
                        if (globe_camera_control._cameraUnderground
                            || globe_camera_control._zoomingUnderground
                            || (camera_position_cartographic.height < 3000.0
                                && (globe_camera_control.direction.dot(cameraPositionNormal))
                                    .abs()
                                    < 0.6))
                        {
                            zoomOnVector = true;
                        } else {
                            let centerPixel = Vec2::ZERO;
                            centerPixel.x = window_size.x / 2.;
                            centerPixel.y = window_size.y / 2.;
                            //TODO: pickEllipsoid取代globe.pick，此刻还没加载地形和模型，所以暂时这么做
                            let centerPosition =
                                globe_camera.pickEllipsoid(&centerPixel, &window_size, &projection);
                            // If centerPosition is not defined, it means the globe does not cover the center position of screen

                            if (centerPosition.is_none()) {
                                zoomOnVector = true;
                            } else if (camera_position_cartographic.height < 1000000.) {
                                // The math in the else block assumes the camera
                                // points toward the earth surface, so we check it here.
                                // Theoretically, we should check for 90 degree, but it doesn't behave well when parallel
                                // to the earth surface
                                if (globe_camera_control.direction.dot(cameraPositionNormal)
                                    >= -0.5)
                                {
                                    zoomOnVector = true;
                                } else {
                                    let cameraPosition =
                                        globe_camera_control.position_cartesian.clone();
                                    let target = globe_camera_control._zoomWorldPosition;

                                    let targetNormal = DVec3::ZERO;

                                    targetNormal = target.normalize();

                                    if (targetNormal.dot(cameraPositionNormal) < 0.0) {
                                        return;
                                    }

                                    let center = DVec3::ZERO;
                                    let forward = DVec3::ZERO;
                                    forward = globe_camera_control.direction.clone();
                                    center = cameraPosition + forward.multiply_by_scalar(1000.);

                                    let positionToTarget = DVec3::ZERO;
                                    let positionToTargetNormal = DVec3::ZERO;
                                    positionToTarget = target.subtract(cameraPosition);

                                    positionToTargetNormal = positionToTarget.normalize();

                                    let alphaDot = cameraPositionNormal.dot(positionToTargetNormal);
                                    if (alphaDot >= 0.0) {
                                        // We zoomed past the target, and this zoom is not valid anymore.
                                        // This line causes the next zoom movement to pick a new starting point.
                                        globe_camera_control._zoomMouseStart.x = -1.0;
                                        return;
                                    }
                                    let alpha = (-alphaDot).acos();
                                    let cameraDistance = cameraPosition.magnitude();
                                    let targetDistance = target.magnitude();
                                    let remainingDistance = cameraDistance - distance;
                                    let positionToTargetDistance = positionToTarget.magnitude();

                                    let gamma = ((positionToTargetDistance / targetDistance)
                                        * alpha.sin())
                                    .clamp(-1.0, 1.0)
                                    .asin();

                                    let delta = ((remainingDistance / targetDistance)
                                        * alpha.sin())
                                    .clamp(-1.0, 1.0)
                                    .asin();

                                    let beta = gamma - delta + alpha;

                                    let mut up = DVec3::ZERO;
                                    up = cameraPosition.normalize();
                                    let right = DVec3::ZERO;
                                    right = positionToTargetNormal.cross(up);
                                    right = right.normalize();

                                    forward = up.cross(right).normalize();

                                    // Calculate new position to move to
                                    center = center
                                        .normalize()
                                        .multiply_by_scalar(center.magnitude() - distance);
                                    cameraPosition = cameraPosition.normalize();
                                    cameraPosition.multiply_by_scalar(remainingDistance);

                                    // Pan
                                    let pMid = DVec3::ZERO;
                                    pMid = (up.multiply_by_scalar(beta.cos() - 1.)
                                        + forward.multiply_by_scalar(beta.sin()))
                                    .multiply_by_scalar(remainingDistance);
                                    cameraPosition = cameraPosition + pMid;

                                    up = center.normalize();
                                    forward = up.cross(right).normalize();

                                    let cMid = DVec3::ZERO;
                                    cMid = (up.multiply_by_scalar(beta.cos() - 1.)
                                        + forward.multiply_by_scalar(beta.sin()))
                                    .multiply_by_scalar(center.magnitude());
                                    center = center + cMid;

                                    // Update camera

                                    // Set new position
                                    globe_camera_control.position_cartesian = cameraPosition;

                                    // Set new direction
                                    globe_camera_control.direction =
                                        center.subtract(cameraPosition).normalize();
                                    globe_camera_control.direction =
                                        globe_camera_control.direction.clone();
                                    // Set new right & up vectors
                                    globe_camera_control.right = globe_camera_control
                                        .direction
                                        .cross(globe_camera_control.up);
                                    globe_camera_control.up = globe_camera_control
                                        .right
                                        .cross(globe_camera_control.direction);

                                    globe_camera.set_view(None, Some(orientation), None, None);
                                    return;
                                }
                            } else {
                                let positionNormal = centerPosition.unwrap().normalize();
                                let pickedNormal =
                                    globe_camera_control._zoomWorldPosition.normalize();
                                let dotProduct = pickedNormal.dot(positionNormal);

                                if (dotProduct > 0.0 && dotProduct < 1.0) {
                                    let angle = acos_clamped(dotProduct);
                                    let axis = pickedNormal.cross(positionNormal);

                                    let denom = {
                                        if angle.abs() > (20.0 as f64).to_radians() {
                                            camera_position_cartographic.height * 0.75
                                        } else {
                                            camera_position_cartographic.height - distance
                                        }
                                    };

                                    let scalar = distance / denom;
                                    globe_camera.rotate(axis, angle * scalar);
                                }
                            }
                        }

                        globe_camera_control._rotatingZoom = !zoomOnVector;
                    }

                    if ((!sameStartPosition && zoomOnVector) || zoomingOnVector) {
                        let ray;
                        let zoomMouseStart = SceneTransforms::wgs84ToWindowCoordinates(
                            &globe_camera_control._zoomWorldPosition,
                            &window_size,
                            &global_transform.compute_matrix(),
                            projection.get_projection_matrix(),
                        );
                        if (startPosition.eq(&globe_camera_control._zoomMouseStart)
                            && zoomMouseStart.is_some())
                        {
                            let v = zoomMouseStart.unwrap();
                            ray = globe_camera
                                .getPickRay(&v, &window_size, &projection)
                                .unwrap();
                        }

                        let rayDirection = ray.direction;

                        globe_camera.move_direction(&rayDirection, distance);

                        globe_camera_control._zoomingOnVector = true;
                    } else {
                        globe_camera.zoom_in(Some(distance));
                    }

                    if (!globe_camera_control._cameraUnderground) {
                        globe_camera.set_view(None, Some(orientation), None, None);
                    }
                }

                ControlEvent::Spin(data) => {}

                ControlEvent::Tilt(data) => {}
            }
        }
    }
}
// pub fn getZoomDistanceUnderground(
//     ellipsoid: &Ellipsoid,
//     ray: houtu_scene::Ray,
//     camera: &GlobeCameraControl,
// ) -> f64 {
//     let origin = ray.origin;
//     let direction = ray.direction;
//     let distanceFromSurface = getDistanceFromSurface(camera);

//     // Weight zoom distance based on how strongly the pick ray is pointing inward.
//     // Geocentric normal is accurate enough for these purposes
//     let surfaceNormal = origin.normalize();
//     let mut strength = (surfaceNormal.dot(direction)).abs();
//     strength = strength.max(0.5) * 2.0;
//     return distanceFromSurface * strength;
// }
// pub fn getDistanceFromSurface(camera: &GlobeCameraControl) -> f64 {
//     let mut height = 0.0;
//     let cartographic =
//         Ellipsoid::WGS84.cartesianToCartographic(&globe_camera_control.position_cartesian);
//     if let Some(v) = cartographic {
//         height = v.height;
//     }
//     let globeHeight = 0.;
//     let distanceFromSurface = (globeHeight - height).abs();
//     return distanceFromSurface;
// }
// pub fn pickGlobe(camera:&GlobeCameraControl, mousePosition:Vec2) ->DVec3{
//     let scene = controller._scene;
//     let globe = controller._globe;
//     let camera = scene.camera;

//     if (!defined(globe)) {
//       return undefined;
//     }

//     let cullBackFaces = !globe_camera_control._cameraUnderground;

//     let depthIntersection;
//     if (scene.pickPositionSupported) {
//       depthIntersection = scene.pickPositionWorldCoordinates(
//         mousePosition,
//         scratchDepthIntersection
//       );
//     }

//     let ray = globe_camera_control.getPickRay(mousePosition, pickGlobeScratchRay);
//     let rayIntersection = globe.pickWorldCoordinates(
//       ray,
//       scene,
//       cullBackFaces,
//       scratchRayIntersection
//     );

//     let pickDistance = defined(depthIntersection)
//       ? Cartesian3.distance(depthIntersection, globe_camera_control.positionWC)
//       : Number.POSITIVE_INFINITY;
//     let rayDistance = defined(rayIntersection)
//       ? Cartesian3.distance(rayIntersection, globe_camera_control.positionWC)
//       : Number.POSITIVE_INFINITY;

//     if (pickDistance < rayDistance) {
//       return Cartesian3.clone(depthIntersection, result);
//     }

//     return Cartesian3.clone(rayIntersection, result);
//   }
