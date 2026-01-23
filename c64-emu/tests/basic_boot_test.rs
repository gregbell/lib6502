//! Integration test: Verify C64 boots to BASIC prompt.
//!
//! This test loads real ROMs and runs the emulator until BASIC initializes.

use c64_emu::{C64System, Region};
use lib6502::Device;
use std::fs;
use std::path::PathBuf;

fn get_rom_path(name: &str) -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join("Downloads").join(name)
}

fn load_rom(name: &str) -> Vec<u8> {
    let path = get_rom_path(name);
    fs::read(&path).unwrap_or_else(|e| panic!("Failed to load ROM {}: {}", path.display(), e))
}

/// Read screen memory and convert to string.
fn read_screen_text(c64: &mut C64System, start: u16, len: u16) -> String {
    let mut text = String::new();
    for i in 0..len {
        let byte = c64.read_memory(start + i);
        // Convert C64 screen codes to ASCII
        let ch = match byte {
            0x00 => '@',
            0x01..=0x1A => (byte - 1 + b'A') as char,
            0x20 => ' ',
            0x2E => '.',
            0x30..=0x39 => (byte) as char,
            _ => '?',
        };
        text.push(ch);
    }
    text
}

#[test]
fn test_basic_boot_with_real_roms() {
    // Load ROMs from ~/Downloads
    let basic = load_rom("basic.901226-01.bin");
    let kernal = load_rom("kernal.901227-03.bin");
    let charrom = load_rom("characters.901225-01.bin");

    assert_eq!(basic.len(), 8192, "BASIC ROM should be 8KB");
    assert_eq!(kernal.len(), 8192, "KERNAL ROM should be 8KB");
    assert_eq!(charrom.len(), 4096, "CHARROM should be 4KB");

    // Create C64 system
    let mut c64 = C64System::new(Region::PAL);
    c64.load_roms(&basic, &kernal, &charrom)
        .expect("Failed to load ROMs");
    c64.reset();

    println!("Starting C64 emulation...");
    println!("Reset vector: ${:04X}", c64.pc());

    // Run frames and check for BASIC initialization
    let max_frames = 500; // ~10 seconds of emulation
    let mut basic_started = false;

    for frame in 0..max_frames {
        c64.step_frame();

        // Check BASIC start pointer at $2B-$2C
        let basic_start_lo = c64.read_memory(0x002B);
        let basic_start_hi = c64.read_memory(0x002C);
        let basic_start = (basic_start_hi as u16) << 8 | basic_start_lo as u16;

        if basic_start != 0 && !basic_started {
            basic_started = true;
            println!(
                "Frame {}: BASIC started! basicStart = ${:04X}",
                frame, basic_start
            );
        }

        // Every 50 frames, print status
        if frame % 50 == 0 {
            let pc = c64.pc();

            // Check CIA1 ICR to verify our fix
            let cia1_icr = c64.read_memory(0xDC0D);

            println!(
                "Frame {}: PC=${:04X}, basicStart=${:04X}, CIA1_ICR=${:02X}",
                frame, pc, basic_start, cia1_icr
            );

            // Read first line of screen memory ($0400)
            let screen_line = read_screen_text(&mut c64, 0x0400, 40);
            println!("  Screen: [{}]", screen_line.trim());
        }

        // Check if we see "READY." on screen (typically around line 5-6)
        // Screen memory starts at $0400, 40 chars per line
        for line in 0..25 {
            let line_addr = 0x0400 + (line * 40);
            let line_text = read_screen_text(&mut c64, line_addr, 40);
            if line_text.contains("READY") {
                println!("\n=== SUCCESS! Found READY. at frame {} ===", frame);
                println!("Screen line {}: [{}]", line, line_text.trim());

                // Print full screen for verification
                println!("\n=== Screen Contents ===");
                for l in 0..25 {
                    let addr = 0x0400 + (l * 40);
                    let text = read_screen_text(&mut c64, addr, 40);
                    println!("{:02}: [{}]", l, text);
                }

                return; // Test passed!
            }
        }
    }

    // If we get here, BASIC didn't boot
    println!(
        "\n=== FAILED: BASIC did not boot after {} frames ===",
        max_frames
    );

    // Print final state for debugging
    let pc = c64.pc();
    let basic_start_lo = c64.read_memory(0x002B);
    let basic_start_hi = c64.read_memory(0x002C);
    let basic_start = (basic_start_hi as u16) << 8 | basic_start_lo as u16;

    println!("Final PC: ${:04X}", pc);
    println!("Final basicStart: ${:04X}", basic_start);

    // Print screen
    println!("\n=== Final Screen Contents ===");
    for l in 0..25 {
        let addr = 0x0400 + (l * 40);
        let text = read_screen_text(&mut c64, addr, 40);
        println!("{:02}: [{}]", l, text);
    }

    panic!("BASIC did not boot - see output above for diagnostics");
}

#[test]
fn test_cia_icr_clears_on_read() {
    // This test verifies our CIA fix: reading ICR should clear flags
    let basic = load_rom("basic.901226-01.bin");
    let kernal = load_rom("kernal.901227-03.bin");
    let charrom = load_rom("characters.901225-01.bin");

    let mut c64 = C64System::new(Region::PAL);
    c64.load_roms(&basic, &kernal, &charrom)
        .expect("Failed to load ROMs");
    c64.reset();

    // Run a few frames to let timers generate interrupts
    for _ in 0..10 {
        c64.step_frame();
    }

    // Read CIA1 ICR twice - second read should be 0
    let icr1 = c64.read_memory(0xDC0D);
    let icr2 = c64.read_memory(0xDC0D);

    println!("CIA1 ICR first read:  ${:02X}", icr1);
    println!("CIA1 ICR second read: ${:02X}", icr2);

    // Second read should be 0 (flags cleared by first read)
    assert_eq!(icr2, 0, "CIA ICR should be cleared after first read");
}

#[test]
fn test_framebuffer_has_characters() {
    // This test verifies VIC-II is rendering characters to the framebuffer
    let basic = load_rom("basic.901226-01.bin");
    let kernal = load_rom("kernal.901227-03.bin");
    let charrom = load_rom("characters.901225-01.bin");

    let mut c64 = C64System::new(Region::PAL);
    c64.load_roms(&basic, &kernal, &charrom)
        .expect("Failed to load ROMs");
    c64.reset();

    // Run until BASIC boots
    for _ in 0..150 {
        c64.step_frame();
    }

    // Get framebuffer
    let fb = c64.get_framebuffer_flat();
    println!("Framebuffer size: {} bytes", fb.len());

    // Check VIC-II state
    let vic_regs = c64.get_vic_registers();
    println!("VIC-II registers:");
    println!(
        "  $D011 (CR1): ${:02X} - DEN={}",
        vic_regs[0x11],
        (vic_regs[0x11] & 0x10) != 0
    );
    println!("  $D016 (CR2): ${:02X}", vic_regs[0x16]);
    println!("  $D018 (Mem): ${:02X}", vic_regs[0x18]);
    println!("  $D020 (Border): ${:02X}", vic_regs[0x20]);
    println!("  $D021 (BG0): ${:02X}", vic_regs[0x21]);

    // Check bank config
    let (basic_on, kernal_on, charrom_on, port01) = c64.get_bank_config();
    println!(
        "Bank config: BASIC={}, KERNAL={}, CHARROM={}, Port01=${:02X}",
        basic_on, kernal_on, charrom_on, port01
    );

    // Check screen memory ($0400) - first few characters
    println!("\nScreen RAM at $0400:");
    for row in 0..5 {
        print!("  Row {}: ", row);
        for col in 0..10 {
            let addr = 0x0400 + row * 40 + col;
            print!("${:02X} ", c64.read_memory(addr));
        }
        println!();
    }

    // Check color RAM ($D800)
    println!("\nColor RAM at $D800:");
    for row in 0..5 {
        print!("  Row {}: ", row);
        for col in 0..10 {
            let addr = 0xD800 + row * 40 + col;
            print!("${:02X} ", c64.read_memory(addr));
        }
        println!();
    }

    // Count unique colors in framebuffer
    let mut color_counts = [0u32; 16];
    for &pixel in fb.iter() {
        if pixel < 16 {
            color_counts[pixel as usize] += 1;
        }
    }

    println!("\nColor distribution in framebuffer:");
    for (color, count) in color_counts.iter().enumerate() {
        if *count > 0 {
            println!("  Color {}: {} pixels", color, count);
        }
    }

    // Check if there's more than just background color
    // Note: C64 default text color is light blue (14), same as border!
    // So we should only exclude background color when counting text pixels.
    let border_color = vic_regs[0x20] & 0x0F;
    let bg_color = vic_regs[0x21] & 0x0F;

    // Text color from color RAM (should be 14 = light blue)
    let text_color = c64.read_memory(0xD800) & 0x0F;

    let non_bg_pixels: u32 = color_counts
        .iter()
        .enumerate()
        .filter(|(c, _)| *c != bg_color as usize)
        .map(|(_, count)| count)
        .sum();

    println!(
        "\nBorder color: {}, BG color: {}, Text color: {}",
        border_color, bg_color, text_color
    );
    println!(
        "Non-background pixels (text + any other): {}",
        non_bg_pixels
    );

    // Print first 8 rows of framebuffer for row 0 (first character line)
    println!("\nFramebuffer row 0 (first 80 pixels):");
    for i in 0..80 {
        print!("{:X}", fb[i] & 0x0F);
    }
    println!();

    // There should be text on screen (in light blue color 14)
    // The non-background pixels include both text and potentially other elements
    if non_bg_pixels < 100 {
        println!("\n=== VIC-II Rendering Debug ===");

        // Check $D018 directly from VIC
        let d018 = vic_regs[0x18];
        let char_base = ((d018 >> 1) & 0x07) as u16 * 0x0800;
        let screen_base = ((d018 >> 4) & 0x0F) as u16 * 0x0400;
        println!("$D018: ${:02X}", d018);
        println!("  Character base: ${:04X}", char_base);
        println!("  Screen base: ${:04X}", screen_base);

        // Check VIC bank from CIA2
        let cia2_regs = c64.get_cia2_registers();
        let vic_bank = (!cia2_regs[0]) & 0x03;
        println!(
            "VIC bank: {} (from CIA2 $DD00=${:02X})",
            vic_bank, cia2_regs[0]
        );

        // Check current raster
        let raster = c64.get_current_raster();
        println!("Current raster: {}", raster);

        // Try calling framebuffer() to get the actual buffer reference
        let fb_ptr = c64.get_framebuffer_flat();
        println!("Framebuffer ptr valid, size: {}", fb_ptr.len());

        panic!("VIC-II is not rendering characters to framebuffer");
    }
}

#[test]
fn test_vic_reads_character_rom() {
    // Verify VIC can read character ROM data
    let basic = load_rom("basic.901226-01.bin");
    let kernal = load_rom("kernal.901227-03.bin");
    let charrom = load_rom("characters.901225-01.bin");

    let mut c64 = C64System::new(Region::PAL);
    c64.load_roms(&basic, &kernal, &charrom)
        .expect("Failed to load ROMs");
    c64.reset();

    // The character ROM has known patterns. Let's check the '@' character (code 0).
    // '@' at offset 0 in character ROM, 8 bytes per character.
    // First byte of '@' in the C64 character ROM is $3C (pattern: 00111100)

    // Read directly from character ROM to verify it's loaded
    let mem = c64.memory_mut();
    let char_rom = mem.char_rom();

    println!("Character ROM first 16 bytes:");
    for i in 0..16 {
        print!("${:02X} ", char_rom[i]);
    }
    println!();

    // Now check what VIC sees at $1000 (character ROM in VIC address space)
    // When VIC bank is 0 (default), characters at $1000-$1FFF come from char ROM

    // Read the '@' character pattern (first 8 bytes of char ROM via VIC)
    println!("\nVIC sees at $1000 (via vic_read):");
    for i in 0..8 {
        let byte = mem.vic_read(0x1000 + i as u16);
        print!("${:02X} ", byte);
    }
    println!();

    // The first byte should be $3C for the '@' character
    let first_byte = mem.vic_read(0x1000);
    println!("\nFirst character byte via vic_read: ${:02X}", first_byte);

    // Check VIC bank
    let vic_bank = mem.vic_bank();
    println!("VIC bank: {}", vic_bank);

    // First byte of '@' in C64 char ROM is $3C
    assert_eq!(first_byte, 0x3C, "VIC should see character ROM at $1000");
}

#[test]
fn test_vic_direct_rendering() {
    // Test VIC rendering directly without the full system
    let basic = load_rom("basic.901226-01.bin");
    let kernal = load_rom("kernal.901227-03.bin");
    let charrom = load_rom("characters.901225-01.bin");

    let mut c64 = C64System::new(Region::PAL);
    c64.load_roms(&basic, &kernal, &charrom)
        .expect("Failed to load ROMs");
    c64.reset();

    // Run until BASIC boots
    for _ in 0..150 {
        c64.step_frame();
    }

    // Now manually call the VIC rendering for a visible scanline
    let mem = c64.memory_mut();

    // Prepare test data
    let mut char_data = [0u8; 2048];
    for i in 0..2048 {
        char_data[i] = mem.vic_read(0x1000 + i as u16);
    }

    let mut screen_ram = [0u8; 1000];
    for i in 0..1000 {
        screen_ram[i] = mem.vic_read(0x0400 + i as u16);
    }

    let mut color_ram = [0u8; 1000];
    for i in 0..1000 {
        color_ram[i] = mem.color_ram.read(i as u16) & 0x0F;
    }

    println!("Screen RAM[0..10]: {:?}", &screen_ram[0..10]);
    println!("Color RAM[0..10]: {:?}", &color_ram[0..10]);
    println!(
        "Char data for space ($20): {:?}",
        &char_data[0x20 * 8..0x20 * 8 + 8]
    );
    println!(
        "Char data for '*' ($2A): {:?}",
        &char_data[0x2A * 8..0x2A * 8 + 8]
    );

    // Get VIC state before rendering
    let d011 = mem.vic.read(0x11);
    let d018 = mem.vic.read(0x18);
    println!("$D011: ${:02X}, $D018: ${:02X}", d011, d018);
    println!(
        "DEN: {}, BMM: {}, ECM: {}, MCM: {}",
        (d011 & 0x10) != 0,
        (d011 & 0x20) != 0,
        (d011 & 0x40) != 0,
        mem.vic.read(0x16) & 0x10 != 0
    );

    // Get framebuffer before rendering
    let fb_before = mem.vic.framebuffer().clone();
    println!("Framebuffer[0][0..10] before: {:?}", &fb_before[0][0..10]);

    // Manually call step_scanline for scanline 51 (first visible line)
    mem.vic
        .step_scanline(51, &char_data, &screen_ram, &color_ram);

    // Get framebuffer after rendering
    let fb_after = mem.vic.framebuffer();
    println!(
        "Framebuffer[0][0..10] after scanline 51: {:?}",
        &fb_after[0][0..10]
    );

    // Check if anything changed
    let mut diff_count = 0;
    for x in 0..320 {
        if fb_before[0][x] != fb_after[0][x] {
            diff_count += 1;
        }
    }
    println!("Pixels changed in row 0: {}", diff_count);

    // Render more scanlines
    for scanline in 52..60 {
        mem.vic
            .step_scanline(scanline, &char_data, &screen_ram, &color_ram);
    }

    // Check row 1 of framebuffer (should have part of character row 0)
    let fb_final = mem.vic.framebuffer();
    println!("Framebuffer[0][0..40] final: {:?}", &fb_final[0][0..40]);
    println!("Framebuffer[7][0..40] final: {:?}", &fb_final[7][0..40]);

    // Count non-background pixels
    let bg = mem.vic.read(0x21) & 0x0F;
    let mut non_bg = 0;
    for row in fb_final.iter() {
        for &pixel in row.iter() {
            if pixel != bg {
                non_bg += 1;
            }
        }
    }
    println!("Non-background pixels: {} (bg color: {})", non_bg, bg);
}
