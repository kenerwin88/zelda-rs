use super::*;

#[test]
fn native_poly_runtime_bridge_dual_writes_changes_from_native_state() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[POLY_CONFIG1] = 2;
    ram[POLY_A] = 7;
    ram[POLY_B] = 11;
    write_le_u16(&mut ram, POLY_SHAPE_DEPTH_BIAS, 0x0100);
    ram[POLY_CONFIG_NUM_POLYS] = 3;
    write_le_u16(&mut ram, POLY_TMP0, 0x0201);
    write_le_u16(&mut ram, POLY_X0_FRAC, 0x0010);
    write_le_u16(&mut ram, POLY_X0_STEP, 0x0004);

    let mut runtime = PolyRuntimeState::load_from_ram(&ram);
    {
        let mut bridge = NativePolyRuntimeBridgeMut::new(&mut runtime, &mut ram);
        assert_eq!(bridge.increment_config1(), 3);
        assert_eq!(bridge.subtract_config1(2), 1);
        bridge.set_color_mode(4);
        bridge.set_model(5);
        bridge.add_angles(1, 2);
        bridge.set_base_position(0x30, 0x40);
        bridge.set_shape_depth_bias_low(0x22);
        bridge.set_num_vertices(8);
        assert_eq!(bridge.decrement_num_polys(), 2);
        bridge.set_tmp0_word(0x1234);
        assert_eq!(bridge.decrement_tmp0(), 0x33);
        bridge.set_x0_step(0x0008);
        bridge.add_x0_step_to_fraction();
    }

    assert_eq!(runtime.config1(), 1);
    assert_eq!(runtime.color_mode(), 4);
    assert_eq!(runtime.model(), 5);
    assert_eq!(runtime.angle_a(), 8);
    assert_eq!(runtime.angle_b(), 13);
    assert_eq!(runtime.base_x(), 0x30);
    assert_eq!(runtime.base_y(), 0x40);
    assert_eq!(runtime.shape_depth_bias(), 0x0122);
    assert_eq!(runtime.num_vertices(), 8);
    assert_eq!(runtime.num_polys(), 2);
    assert_eq!(runtime.tmp0_word(), 0x1233);
    assert_eq!(runtime.x0_fraction(), 0x0018);
    assert_eq!(PolyRuntimeState::load_from_ram(&ram), runtime);
}

#[test]
fn native_poly_structured_bridges_dual_write_projection_face_and_edge_state() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[POLY_PROJECTED_X + 3] = 0x11;
    ram[POLY_PROJECTED_Y + 3] = 0x22;
    ram[POLY_FACE_COORDS] = 0x04;
    ram[POLY_FACE_COORDS + 5] = 0xaa;
    ram[POLY_TOTAL_NUM_STEPS] = 0x02;
    ram[POLY_X0_CUR] = 0x10;
    ram[POLY_Y0_CUR] = 0x20;
    ram[POLY_X1_CUR] = 0x30;
    ram[POLY_Y1_CUR] = 0x40;

    let mut projected = PolyProjectedVerticesState::load_from_ram(&ram);
    let mut face = PolyFaceCoordsState::load_from_ram(&ram);
    let mut edge = PolyRasterEdgeState::load_from_ram(&ram);

    {
        let mut bridge = NativePolyProjectedVerticesBridgeMut::new(&mut projected, &mut ram);
        bridge.set_position(3, 0x55, 0x66);
    }
    {
        let mut bridge = NativePolyFaceCoordsBridgeMut::new(&mut face, &mut ram);
        bridge.set_xy_coords_count(0x08);
        bridge.set_coord(5, 0xbb);
    }
    {
        let mut bridge = NativePolyRasterEdgeBridgeMut::new(&mut edge, &mut ram);
        bridge.set_left_current(0x01, 0x02);
        bridge.set_right_target(0x03, 0x04);
        bridge.set_both_cur_vertex_idx(0x09);
        assert_eq!(bridge.decrement_total_num_steps(), 1);
        bridge.increment_y0_cur();
    }

    assert_eq!(projected.x(3), 0x55);
    assert_eq!(projected.y(3), 0x66);
    assert_eq!(face.xy_coords_count(), 0x08);
    assert_eq!(face.coord(5), 0xbb);
    assert_eq!(edge.x0_cur(), 0x01);
    assert_eq!(edge.y0_cur(), 0x03);
    assert_eq!(edge.x1_target(), 0x03);
    assert_eq!(edge.y1_trigger(), 0x04);
    assert_eq!(edge.cur_vertex_idx0(), 0x09);
    assert_eq!(edge.cur_vertex_idx1(), 0x09);
    assert_eq!(edge.total_num_steps(), 1);
    assert_eq!(ram[POLY_PROJECTED_X + 3], 0x55);
    assert_eq!(ram[POLY_PROJECTED_Y + 3], 0x66);
    assert_eq!(ram[POLY_FACE_COORDS], 0x08);
    assert_eq!(ram[POLY_FACE_COORDS + 5], 0xbb);
    assert_eq!(ram[POLY_X0_CUR], 0x01);
    assert_eq!(ram[POLY_Y0_CUR], 0x03);
    assert_eq!(ram[POLY_X1_TARGET], 0x03);
    assert_eq!(ram[POLY_Y1_TRIG], 0x04);
    assert_eq!(ram[POLY_CUR_VERTEX_IDX0], 0x09);
    assert_eq!(ram[POLY_CUR_VERTEX_IDX1], 0x09);
    assert_eq!(ram[POLY_TOTAL_NUM_STEPS], 1);
}
