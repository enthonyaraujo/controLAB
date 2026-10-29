use controlab_engine::presentation;

#[test]
fn exports_real_png_and_svg_with_escaped_labels() {
    let (png,svg)=presentation::plot(&[("G < H",&[0.,1.,2.],&[0.,0.5,1.])],"Título","Tempo (s)","Saída","dark",false).unwrap();
    assert!(png.starts_with("data:image/png;base64,iVBOR"));
    assert!(svg.contains("G &lt; H"));
    assert!(svg.contains("Tempo (s)"));
}
