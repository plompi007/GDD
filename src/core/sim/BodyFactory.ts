import * as RAPIER from '@dimforge/rapier2d-deterministic';
import type { BodySpec, Transform2D } from './BodySpec.ts';

export interface BuiltBody {
  rigidBody: RAPIER.RigidBody;
  collider: RAPIER.Collider;
}

function colliderDescFor(spec: BodySpec): RAPIER.ColliderDesc {
  const desc =
    spec.shape.kind === 'ball'
      ? RAPIER.ColliderDesc.ball(spec.shape.radius)
      : RAPIER.ColliderDesc.cuboid(spec.shape.w / 2, spec.shape.h / 2);

  if (spec.restitution !== undefined) desc.setRestitution(spec.restitution);
  if (spec.friction !== undefined) desc.setFriction(spec.friction);
  if (spec.mass !== undefined) desc.setMass(spec.mass);
  if (spec.isSensor) desc.setSensor(true);
  return desc;
}

/**
 * Builds a rigid body + collider from a data-driven BodySpec. This is the
 * only place PartDef bodies (GDD §5.5.3) turn into real Rapier objects.
 */
export function buildBody(world: RAPIER.World, spec: BodySpec, transform: Transform2D): BuiltBody {
  const rigidBodyDesc =
    spec.type === 'fixed' ? RAPIER.RigidBodyDesc.fixed() : RAPIER.RigidBodyDesc.dynamic();

  rigidBodyDesc.setTranslation(transform.x, transform.y);
  rigidBodyDesc.setRotation(transform.rotation ?? 0);

  if (spec.type === 'dynamic') {
    // Sleeping rigid bodies break determinism in long chains (GDD §2.6.1) —
    // every dynamic body must be kept perpetually awake.
    rigidBodyDesc.setCanSleep(false);
    if (spec.gravityScale !== undefined) rigidBodyDesc.setGravityScale(spec.gravityScale);
    if (spec.linearDamping !== undefined) rigidBodyDesc.setLinearDamping(spec.linearDamping);
    if (spec.angularDamping !== undefined) rigidBodyDesc.setAngularDamping(spec.angularDamping);
    if (spec.ccd) rigidBodyDesc.setCcdEnabled(true);
  }

  const rigidBody = world.createRigidBody(rigidBodyDesc);
  const collider = world.createCollider(colliderDescFor(spec), rigidBody);
  return { rigidBody, collider };
}
