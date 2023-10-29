use sfml::graphics::Transformable;

use crate::visualizer::graphics;

pub(crate) fn clip(
    graphics_context: &graphics::GraphicsContext,
    target: &mut dyn graphics::RenderTarget,
    rect: graphics::FloatRect,
    draw_inner: impl FnOnce(&mut dyn graphics::RenderTarget, graphics::Vector2f),
) {
    let mut sub_graphics =
        graphics::RenderTexture::with_settings(rect.width.ceil() as u32, rect.height.ceil() as u32, &graphics_context.default_render_context_settings).expect("could not create render texture");

    sub_graphics.set_active(true);
    draw_inner(&mut sub_graphics, graphics::Vector2f::new(0.0, 0.0));
    sub_graphics.set_active(false);
    sub_graphics.display();

    let mut sprite = graphics::Sprite::new();
    sprite.set_texture(sub_graphics.texture(), true);
    sprite.set_position((rect.left, rect.top));
    target.draw(&sprite);
}
