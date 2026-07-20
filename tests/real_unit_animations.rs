use std::path::PathBuf;

use pop3::data::animation::{
    build_direct_multi_anim_atlas, build_multi_anim_atlas, unit_combo_for_subtype,
    AnimationSequence, AnimationsData, SHAMAN_ANIMS, SHAMAN_RUNTIME_COMPOSITED_ANIMS,
    UNIT_RUNTIME_ANIMS,
};
use pop3::data::psfb::ContainerPSFB;
use pop3::data::types::BinDeserializer;
use pop3::render::sprites::convert_palette;

fn original_data() -> PathBuf {
    PathBuf::from(
        std::env::var("POP3_DATA_DIR")
            .expect("POP3_DATA_DIR must point to the legally owned original game"),
    )
}

fn opaque_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4).filter(|pixel| pixel[3] != 0).count()
}

#[test]
#[ignore = "requires legally owned POP3_DATA_DIR assets"]
fn runtime_unit_atlases_contain_every_supported_native_action() {
    let base = original_data();
    let data = base.join("data");
    let sprites = ContainerPSFB::from_file(&data.join("HSPR0-0.DAT"))
        .expect("original HSPR person bank must decode");
    let palette = convert_palette(&std::fs::read(data.join("pal0-0.dat")).unwrap());
    let sequences = AnimationSequence::from_data(&AnimationsData::from_path(&data));

    for &(subtype, animations) in &UNIT_RUNTIME_ANIMS {
        let (_, _, rgba, _, _, _, offsets, _) = build_multi_anim_atlas(
            &sequences,
            &sprites,
            &palette,
            animations,
            unit_combo_for_subtype(subtype),
        )
        .unwrap_or_else(|| panic!("subtype {subtype} runtime atlas must build"));
        let packed = offsets
            .iter()
            .map(|(animation, _, _)| *animation)
            .collect::<Vec<_>>();
        assert_eq!(packed, animations, "subtype {subtype} lost an action row");
        assert!(opaque_pixels(&rgba) > 0);
    }

    let (_, _, direct_rgba, _, _, _, direct_offsets, _) =
        build_direct_multi_anim_atlas(&sprites, &palette, &SHAMAN_ANIMS)
            .expect("direct shaman idle/walk atlas must build");
    assert_eq!(
        direct_offsets
            .iter()
            .map(|(animation, _, _)| *animation)
            .collect::<Vec<_>>(),
        vec![20, 26]
    );
    assert!(opaque_pixels(&direct_rgba) > 0);

    let (_, _, action_rgba, _, _, _, action_offsets, _) = build_multi_anim_atlas(
        &sequences,
        &sprites,
        &palette,
        SHAMAN_RUNTIME_COMPOSITED_ANIMS,
        None,
    )
    .expect("composited shaman action atlas must build");
    assert_eq!(
        action_offsets
            .iter()
            .map(|(animation, _, _)| *animation)
            .collect::<Vec<_>>(),
        SHAMAN_RUNTIME_COMPOSITED_ANIMS
    );
    assert!(opaque_pixels(&action_rgba) > 0);
}

#[test]
#[ignore = "requires legally owned POP3_DATA_DIR assets"]
fn specialist_runtime_atlas_keeps_native_type_layers() {
    let base = original_data();
    let data = base.join("data");
    let sprites = ContainerPSFB::from_file(&data.join("HSPR0-0.DAT")).unwrap();
    let palette = convert_palette(&std::fs::read(data.join("pal0-0.dat")).unwrap());
    let sequences = AnimationSequence::from_data(&AnimationsData::from_path(&data));

    let (_, _, complete, _, _, _, _, _) = build_multi_anim_atlas(
        &sequences,
        &sprites,
        &palette,
        &[16],
        unit_combo_for_subtype(3),
    )
    .unwrap();
    let (_, _, common_only, _, _, _, _, _) =
        build_multi_anim_atlas(&sequences, &sprites, &palette, &[16], None).unwrap();

    assert_ne!(complete, common_only);
    assert!(
        opaque_pixels(&complete) > opaque_pixels(&common_only),
        "the warrior helmet/armor layer must add visible pixels"
    );
}
