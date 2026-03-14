"""Generate all icon PNGs from SVG sources using cairosvg."""

import os
import shutil

import cairosvg

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
TAURI_ICONS = os.path.join(SCRIPT_DIR, '..', 'src-tauri', 'icons')


def svg_to_png(svg_path, out_path, size):
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    cairosvg.svg2png(url=svg_path, write_to=out_path,
                     output_width=size, output_height=size)
    print(f"  {os.path.relpath(out_path, SCRIPT_DIR)} ({size}x{size})")


def main():
    icon_svg = os.path.join(SCRIPT_DIR, 'icon.svg')
    tray_svg = os.path.join(SCRIPT_DIR, 'tray-icon.svg')

    print("Tauri app icons:")
    tauri_sizes = {
        '32x32.png': 32,
        '128x128.png': 128,
        '128x128@2x.png': 256,
        'icon.png': 512,
        'Square30x30Logo.png': 30,
        'Square44x44Logo.png': 44,
        'Square71x71Logo.png': 71,
        'Square89x89Logo.png': 89,
        'Square107x107Logo.png': 107,
        'Square142x142Logo.png': 142,
        'Square150x150Logo.png': 150,
        'Square284x284Logo.png': 284,
        'Square310x310Logo.png': 310,
        'StoreLogo.png': 50,
    }
    for filename, size in tauri_sizes.items():
        svg_to_png(icon_svg, os.path.join(TAURI_ICONS, filename), size)

    print("\nTray icon:")
    svg_to_png(tray_svg, os.path.join(TAURI_ICONS, 'tray-icon.png'), 44)

    # Generate .ico for Windows (multi-size) using Pillow
    print("\nWindows .ico:")
    from PIL import Image
    from io import BytesIO

    ico_sizes = [16, 32, 48, 256]
    ico_images = []
    for size in ico_sizes:
        buf = BytesIO()
        cairosvg.svg2png(url=icon_svg, write_to=buf,
                         output_width=size, output_height=size)
        buf.seek(0)
        ico_images.append(Image.open(buf).convert('RGBA'))

    ico_path = os.path.join(TAURI_ICONS, 'icon.ico')
    ico_images[0].save(ico_path, format='ICO',
                       sizes=[(s, s) for s in ico_sizes],
                       append_images=ico_images[1:])
    print(f"  {os.path.relpath(ico_path, SCRIPT_DIR)}")

    # Generate .icns for macOS via iconutil
    print("\nmacOS .icns:")
    iconset_dir = os.path.join(TAURI_ICONS, 'icon.iconset')
    os.makedirs(iconset_dir, exist_ok=True)

    icns_sizes = {
        'icon_16x16.png': 16,
        'icon_16x16@2x.png': 32,
        'icon_32x32.png': 32,
        'icon_32x32@2x.png': 64,
        'icon_128x128.png': 128,
        'icon_128x128@2x.png': 256,
        'icon_256x256.png': 256,
        'icon_256x256@2x.png': 512,
        'icon_512x512.png': 512,
        'icon_512x512@2x.png': 1024,
    }
    for filename, size in icns_sizes.items():
        svg_to_png(icon_svg, os.path.join(iconset_dir, filename), size)

    icns_path = os.path.join(TAURI_ICONS, 'icon.icns')
    os.system(f'iconutil -c icns -o "{icns_path}" "{iconset_dir}"')
    shutil.rmtree(iconset_dir)
    print(f"  {os.path.relpath(icns_path, SCRIPT_DIR)}")

    print("\nDone!")


if __name__ == '__main__':
    main()
