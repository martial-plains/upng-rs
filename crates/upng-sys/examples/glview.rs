#![no_std]
#![no_main]

use core::{ffi::c_void, mem, ptr};

use glu_sys::{
    glBegin, glBindTexture, glBlendFunc, glClear, glClearColor, glDeleteTextures, glDisable,
    glEnable, glEnd, glGenTextures, glLoadIdentity, glMatrixMode, glOrtho, glTexCoord2f,
    glTexImage2D, glTexParameteri, glVertex2f, GLint, GLsizei, GLuint, GLvoid, GL_BLEND,
    GL_COLOR_BUFFER_BIT, GL_CULL_FACE, GL_DEPTH_TEST, GL_LINEAR, GL_LUMINANCE, GL_LUMINANCE_ALPHA,
    GL_MODELVIEW, GL_ONE_MINUS_SRC_ALPHA, GL_PROJECTION, GL_QUADS, GL_RGB, GL_RGBA, GL_SRC_ALPHA,
    GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_TEXTURE_MIN_FILTER, GL_UNSIGNED_BYTE,
};
use libc::{c_char, c_int, c_uchar, calloc, free, printf};

use sdl::{gl::ll::SDL_GL_SwapBuffers, video::ll::SDL_SetVideoMode};
use sdl2_sys::{SDL_Event, SDL_EventType, SDL_Init, SDL_Quit, SDL_WaitEvent, SDL_INIT_VIDEO};
use upng_sys::{
    upng_decode, upng_free, upng_get_buffer, upng_get_components, upng_get_error,
    upng_get_error_line, upng_get_height, upng_get_width, upng_new_from_file, upng_t, UPNG_EOK,
};

pub const SDL_OPENGL: u32 = 2;
pub const SDL_DOUBLEBUF: u32 = 1_073_741_824;

fn checkboard(w: GLuint, h: GLuint) -> GLuint {
    let mut xc = 0;
    let mut dark = 0;
    let mut texture = 0;

    let buffer: *mut c_uchar = unsafe { calloc((w * h) as usize, 3).cast::<c_uchar>() };

    unsafe { printf(c"%i %i\n".as_ptr(), w, h) };
    for y in 0..=h {
        for x in 0..=w {
            xc += 1;

            if (xc % (w >> 3)) == 0 {
                dark = 1 - dark;
            }

            if dark != 0 {
                unsafe {
                    buffer.add((y * w * 3 + x * 3) as usize).write(0x6F);
                    buffer.add((y * w * 3 + x * 3 + 1) as usize).write(0x6F);
                    buffer.add((y * w * 3 + x * 3 + 2) as usize).write(0x6F);
                }
            } else {
                unsafe {
                    buffer.add((y * w * 3 + x * 3) as usize).write(0xAF);
                    buffer.add((y * w * 3 + x * 3 + 1) as usize).write(0xAF);
                    buffer.add((y * w * 3 + x * 3 + 2) as usize).write(0xAF);
                }
            }
        }

        if (y % (h >> 3)) == 0 {
            dark = 1 - dark;
        }
    }

    unsafe {
        glEnable(GL_TEXTURE_2D);
        glGenTextures(1, &raw mut texture);
        glBindTexture(GL_TEXTURE_2D, texture);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as i32);
        glTexImage2D(
            GL_TEXTURE_2D,
            0,
            3,
            w as GLsizei,
            h as GLsizei,
            0,
            GL_RGB,
            GL_UNSIGNED_BYTE,
            buffer as *const GLvoid,
        );

        free(buffer.cast::<c_void>());

        texture
    }
}

#[no_mangle]
unsafe extern "C" fn main(argc: c_int, argv: *const *const c_char) -> c_int {
    if argc <= 1 {
        return 0;
    }

    let upng = load_image(argv.add(1).read());
    if upng.is_null() {
        return 0;
    }

    setup_sdl(upng);

    let texture = create_texture(upng);
    if texture == 0 {
        return 1;
    }

    let cb = checkboard(upng_get_width(upng), upng_get_height(upng));

    let mut event: SDL_Event = mem::zeroed();
    run_event_loop(&mut event, texture, cb);

    glDeleteTextures(1, &texture);
    glDeleteTextures(1, &cb);
    SDL_Quit();

    0
}

unsafe fn load_image(file: *const c_char) -> *mut upng_t {
    let upng = upng_new_from_file(file);
    upng_decode(upng);

    if upng_get_error(upng) != UPNG_EOK {
        printf(
            c"error: %u %u\n".as_ptr(),
            upng_get_error(upng),
            upng_get_error_line(upng),
        );
        upng_free(upng);
        return ptr::null_mut();
    }

    upng
}

unsafe fn create_texture(upng: *const upng_t) -> GLuint {
    let mut texture: GLuint = 0;

    glEnable(GL_TEXTURE_2D);
    glGenTextures(1, &raw mut texture);

    glBindTexture(GL_TEXTURE_2D, texture);

    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as GLint);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as GLint);

    let width = upng_get_width(upng) as GLsizei;
    let height = upng_get_height(upng) as GLsizei;
    let buffer = upng_get_buffer(upng).cast::<c_void>();

    match upng_get_components(upng) {
        1 => glTexImage2D(
            GL_TEXTURE_2D,
            0,
            GL_LUMINANCE as GLint,
            width,
            height,
            0,
            GL_LUMINANCE,
            GL_UNSIGNED_BYTE,
            buffer,
        ),
        2 => glTexImage2D(
            GL_TEXTURE_2D,
            0,
            GL_LUMINANCE_ALPHA as GLint,
            width,
            height,
            0,
            GL_LUMINANCE_ALPHA,
            GL_UNSIGNED_BYTE,
            buffer,
        ),
        3 => glTexImage2D(
            GL_TEXTURE_2D,
            0,
            GL_RGB as GLint,
            width,
            height,
            0,
            GL_RGB,
            GL_UNSIGNED_BYTE,
            buffer,
        ),
        4 => glTexImage2D(
            GL_TEXTURE_2D,
            0,
            GL_RGBA as GLint,
            width,
            height,
            0,
            GL_RGBA,
            GL_UNSIGNED_BYTE,
            buffer,
        ),
        _ => return 0,
    };

    texture
}

unsafe fn setup_sdl(upng: *const upng_t) {
    SDL_Init(SDL_INIT_VIDEO);
    SDL_SetVideoMode(
        upng_get_width(upng) as c_int,
        upng_get_height(upng) as c_int,
        0,
        SDL_OPENGL | SDL_DOUBLEBUF,
    );

    glDisable(GL_DEPTH_TEST);
    glDisable(GL_CULL_FACE);
    glEnable(GL_BLEND);
    glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
    glClearColor(0.0, 0.0, 0.0, 0.0);

    glMatrixMode(GL_PROJECTION);
    glLoadIdentity();
    glOrtho(0.0, 1.0, 0.0, 1.0, 0.0, 1.0);

    glMatrixMode(GL_MODELVIEW);
    glLoadIdentity();
}

unsafe fn run_event_loop(event: &mut SDL_Event, texture: GLuint, cb: GLuint) {
    while SDL_WaitEvent(event) != 0 {
        if event.type_ == SDL_EventType::SDL_QUIT as u32 {
            break;
        }

        render_frame(texture, cb);
    }
}

unsafe fn render_frame(texture: GLuint, cb: GLuint) {
    glClear(GL_COLOR_BUFFER_BIT);

    draw_texture(cb);
    draw_texture(texture);

    SDL_GL_SwapBuffers();
}

unsafe fn draw_texture(texture: GLuint) {
    glBindTexture(GL_TEXTURE_2D, texture);
    glBegin(GL_QUADS);
    glTexCoord2f(0.0, 1.0);
    glVertex2f(0.0, 0.0);
    glTexCoord2f(0.0, 0.0);
    glVertex2f(0.0, 1.0);
    glTexCoord2f(1.0, 0.0);
    glVertex2f(1.0, 1.0);
    glTexCoord2f(1.0, 1.0);
    glVertex2f(1.0, 0.0);
    glEnd();
}
