// Farmer's Delight's steam (`ModParticleTypes.STEAM`, `SteamParticle`): a
// `TextureSheetParticle` with one of the twelve `steam_N` sprites (copied
// under the particle textures as `farmersdelight_steam_N`), drawn on the
// translucent layer, ticked by its own `tick` (`tick`, from the engine's
// per-particle tick).
use super::*;

/// The sprite set (`particles/steam.json`).
const SPRITES: u32 = 12;
const PREFIX: &str = "particle/farmersdelight_steam_";

/// Whether `p` is a steam particle.
pub fn is_steam(p: &Particle) -> bool {
    matches!(&p.sprite, Sprite::Named(name) if name.starts_with(PREFIX))
}

/// `SteamParticle.Factory.createParticle`: the particle 0.3 higher, at
/// alpha 0.6, with a random sprite of the set.
pub fn add(engine: &mut ParticleEngine, at: [f64; 3], motion: [f64; 3]) {
    let c = engine.camera;
    let d2 = (at[0] - c[0]).powi(2) + (at[1] - c[1]).powi(2) + (at[2] - c[2]).powi(2);
    // ClientLevel.addParticle: within 32 blocks, unless particles are minimal.
    if d2 > 1024.0 || engine.particle_level >= 2 {
        return;
    }
    let seed = engine.seeds.next_long();
    // Particle(level, x, y, z) and SingleQuadParticle's quadSize.
    let mut p = Particle::at(&mut engine.random, ParticleType::WhiteSmoke, [at[0], at[1] + 0.3, at[2]]);
    p.random = LegacyRandom::new(seed);
    // scale(2.0F), then setSize(0.25F, 0.25F).
    p.size *= 2.0;
    p.bb_width = 0.25;
    p.bb_height = 0.25;
    p.bb = aabb_at(p.position, 0.25, 0.25);
    p.lifetime = p.random.next_int_bound(50).unwrap_or(0) + 80;
    p.gravity = 3.0e-6;
    p.velocity = [
        motion[0],
        motion[1] + f64::from(p.random.next_float() / 500.0),
        motion[2],
    ];
    p.rgba[3] = 0.6;
    let n = p.random.next_int_bound(SPRITES as i32).unwrap_or(0);
    p.sprite = Sprite::Named(format!("{PREFIX}{n}"));
    engine.push(p);
}

/// `SteamParticle.tick`: a drift that wanders a little, the faint
/// gravity, and the last 60 ticks fading out.
pub fn tick(p: &mut Particle, level: &ParticleLevel) {
    p.previous = p.position;
    let age = p.age;
    p.age += 1;
    if age < p.lifetime && p.rgba[3] > 0.0 {
        let sign = |r: &mut LegacyRandom| if r.next_boolean() { 1.0 } else { -1.0 };
        let dx = p.random.next_float() / 5000.0 * sign(&mut p.random);
        p.velocity[0] += f64::from(dx);
        let dz = p.random.next_float() / 5000.0 * sign(&mut p.random);
        p.velocity[2] += f64::from(dz);
        p.velocity[1] -= f64::from(p.gravity);
        p.move_by(level, p.velocity);
        if p.age >= p.lifetime - 60 && p.rgba[3] > 0.01 {
            p.rgba[3] -= 0.02;
        }
    } else {
        p.removed = true;
    }
}
