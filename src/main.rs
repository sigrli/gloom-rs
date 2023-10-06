// Uncomment these following global attributes to silence most warnings of "low" interest:
/*
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]
*/
extern crate nalgebra_glm as glm;
use std::{ mem, ptr, os::raw::c_void };
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

mod shader;
mod util;
mod mesh;
mod scene_graph;
use scene_graph::SceneNode;
mod toolbox;
use toolbox::Heading;

use gl::{GetUniformLocation, GetSubroutineUniformLocation, BindVertexArray};
use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;

// initial window size
const INITIAL_SCREEN_W: u32 = 800;
const INITIAL_SCREEN_H: u32 = 600;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //

// Get the size of an arbitrary array of numbers measured in bytes
// Example usage:  pointer_to_array(my_array)
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}
// Get the OpenGL-compatible pointer to an arbitrary array of numbers
// Example usage:  pointer_to_array(my_array)
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}
// Get the size of the given type in bytes
// Example usage:  size_of::<u64>()
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}
// Get an offset in bytes for n units of type T, represented as a relative pointer
// Example usage:  offset::<u64>(4)
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}
// Get a null pointer (equivalent to an offset of 0)
// ptr::null()


// == // Generate your VAO here
unsafe fn create_vao(vertices: &Vec<f32>, indices: &Vec<u32>, colors: &Vec<f32>, normal_vec: &Vec<f32>,) -> u32 {
    // This should:
    // * Generate a VAO and bind it
    let mut vao = 0;
    gl::GenVertexArrays(1, &mut vao);
    gl::BindVertexArray(vao);
    

    // * Generate a VBO and bind it
    let mut vbo=0;
    gl::GenBuffers(1, &mut vbo);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

    // * Fill it with data
    gl::BufferData(gl::ARRAY_BUFFER, byte_size_of_array(vertices), pointer_to_array(vertices), gl::STATIC_DRAW);
    
    // * Configure a VAP for the data and enable it
    //If your buffer only contains a single entry type, you can pass in 0 for stride
    gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
    gl::EnableVertexAttribArray(0);
   


    // * Generate a IBO and bind it
    let mut ibo =0;
    gl::GenBuffers(1, &mut ibo);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);

    // * Fill it with data
    gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, byte_size_of_array(indices), pointer_to_array(indices), gl::STATIC_DRAW);

    
    
    // Create a VBO and bind it for color 
    let mut vbo_color = 0;
    gl::GenBuffers(1, &mut vbo_color);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo_color);
    //Fill it with data
    gl::BufferData(gl::ARRAY_BUFFER, byte_size_of_array(colors), pointer_to_array(colors), gl::STATIC_DRAW);

    // Configure a VAP for the color and enable it
    gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, 0, ptr::null());
    gl::EnableVertexAttribArray(1);



    //include normal vectors mesh (terrain)
    let mut vbo_mesh = 0;
    gl::GenBuffers(1, &mut vbo_mesh);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo_mesh);
    gl::BufferData(gl::ARRAY_BUFFER, byte_size_of_array(normal_vec), pointer_to_array(normal_vec), gl::STATIC_DRAW);
    // Configure a VAP for the normal vector and enable it
    gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
    gl::EnableVertexAttribArray(2);

    //helicopter
    let mut vbo_heli = 0;
    gl::GenBuffers(1, &mut vbo_heli);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo_heli);
    gl::BufferData(gl::ARRAY_BUFFER, byte_size_of_array(normal_vec), pointer_to_array(normal_vec), gl::STATIC_DRAW);
    // Configure a VAP for the helicopter and enable it
    gl::VertexAttribPointer(3, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
    gl::EnableVertexAttribArray(3);

    // * Return the ID of the VAO
    vao
}



fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize::new(INITIAL_SCREEN_W, INITIAL_SCREEN_H));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Set up shared tuple for tracking changes to the window size
    let arc_window_size = Arc::new(Mutex::new((INITIAL_SCREEN_W, INITIAL_SCREEN_H, false)));
    // Make a reference of this tuple to send to the render thread
    let window_size = Arc::clone(&arc_window_size);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers.
        // This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        let mut window_aspect_ratio = INITIAL_SCREEN_W as f32 / INITIAL_SCREEN_H as f32;

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!("{}: {}", util::get_gl_string(gl::VENDOR), util::get_gl_string(gl::RENDERER));
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!("GLSL\t: {}", util::get_gl_string(gl::SHADING_LANGUAGE_VERSION));
        }

        // == // Set up your VAO around here

        let mesh = mesh::Terrain::load("./resources/lunarsurface.obj");
        //load helicopter
        let helicopter = mesh::Helicopter::load("./resources/helicopter.obj"); 
            
        // Create the VAO for terrain
        let my_vao = unsafe { create_vao(&mesh.vertices, &mesh.indices, &mesh.colors, &mesh.normals) };
        //Create the vao for the helicopter
        let vao_body = unsafe{create_vao(&helicopter.body.vertices, &helicopter.body.indices, &helicopter.body.colors, &helicopter.body.normals)};
        let vao_door = unsafe {create_vao(&helicopter.door.vertices, &helicopter.door.indices, &helicopter.door.colors, &helicopter.door.normals)};
        let vao_main_rotor = unsafe{create_vao(&helicopter.main_rotor.vertices, &helicopter.main_rotor.indices, &helicopter.main_rotor.colors, &helicopter.main_rotor.normals)};
        let vao_tail_rotor = unsafe{create_vao(&helicopter.tail_rotor.vertices, &helicopter.tail_rotor.indices, &helicopter.tail_rotor.colors, &helicopter.tail_rotor.normals)};

        //Assingment 3 task 2b) == Scene Nodes

        //root node for the scene
        let mut root_node = SceneNode::new();

        //SceneNode for terrain
        let mut terrain_node = SceneNode::from_vao(my_vao, mesh.indices.len() as i32);
                
        //SceneNode for helicopter
        //let mut helicopter_root_node = SceneNode::new();
        
        //helicopter debug?
        //helicopter_root_node.print();

        
        //TASK 6
        // Create a vector to store helicopter scene nodes
        let mut helicopters: Vec<scene_graph::Node> = Vec::new();
        // Create a vector to store terrain scene nodes
        //let mut terrains: Vec<scene_graph::Node> = Vec::new();
        
        // Instantiate 5 helicopters and terrains
        for i in 0..5 {
            //SceneNode for each helicopter part
            let mut helicopter_body_node = SceneNode::from_vao(vao_body, helicopter.body.indices.len() as i32);
            let mut helicopter_door_node = SceneNode::from_vao(vao_door, helicopter.door.indices.len() as i32);
            let mut helicopter_mainrotor_node = SceneNode::from_vao(vao_main_rotor, helicopter.main_rotor.indices.len() as i32);
            let mut helicopter_tailrotor_node = SceneNode::from_vao(vao_tail_rotor, helicopter.tail_rotor.indices.len() as i32);
            
            // Define the reference point for the tail rotor
            helicopter_tailrotor_node.reference_point = glm::vec3(0.35, 2.3, 10.4);

            terrain_node.add_child(&helicopter_body_node);

            //attach helicopter nodes to the root for the helicopter
            helicopter_body_node.add_child(&helicopter_door_node);
            helicopter_body_node.add_child(&helicopter_mainrotor_node);
            helicopter_body_node.add_child(&helicopter_tailrotor_node);

            helicopters.push(helicopter_body_node);

        
        }    
        //attach terrain node to the root node
        root_node.add_child(&terrain_node);


        // == // Set up your shaders here

        // Basic usage of shader helper:
        // The example code below creates a 'shader' object.
        // It which contains the field `.program_id` and the method `.activate()`.
        // The `.` in the path is relative to `Cargo.toml`.
        // This snippet is not enough to do the exercise, and will need to be modified (outside
        // of just using the correct path), but it only needs to be called once

        
        let simple_shader = unsafe {
            shader::ShaderBuilder::new()
                .attach_file("./shaders/simple.vert")
                .attach_file("./shaders/simple.frag")
                .link()
        };
        unsafe { simple_shader.activate() };



        // Used to demonstrate keyboard handling for exercise 2.
        let mut cam_pos_x  = 0.0;
        let mut cam_pos_y = 0.0;
        let mut cam_pos_z = 0.0;
        let mut cam_rot_x = 0.0;
        let mut cam_rot_y = 0.0;

       

        // The main rendering loop
        let first_frame_time = std::time::Instant::now();
        let mut previous_frame_time = first_frame_time;
        loop {
            // Compute time passed since the previous frame and since the start of the program
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(previous_frame_time).as_secs_f32();
            previous_frame_time = now;

            // Handle resize events
            if let Ok(mut new_size) = window_size.lock() {
                if new_size.2 {
                    context.resize(glutin::dpi::PhysicalSize::new(new_size.0, new_size.1));
                    window_aspect_ratio = new_size.0 as f32 / new_size.1 as f32;
                    (*new_size).2 = false;
                    println!("Window was resized to {}x{}", new_size.0, new_size.1);
                    unsafe { gl::Viewport(0, 0, new_size.0 as i32, new_size.1 as i32); }
                }
            }

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        // The `VirtualKeyCode` enum is defined here:
                        //    https://docs.rs/winit/0.25.0/winit/event/enum.VirtualKeyCode.html

                        VirtualKeyCode::D => {
                            cam_pos_x -= delta_time * 20.0;
                        }
                        VirtualKeyCode::A => {
                            cam_pos_x += delta_time * 20.0;
                        }

                        VirtualKeyCode::W => {
                            cam_pos_z += (delta_time * 20.0) * (-cam_rot_y as f32).cos();
                            cam_pos_x += (delta_time * 20.0) * (-cam_rot_y as f32).sin();
                        }
                        VirtualKeyCode::S => {
                            cam_pos_z -= (delta_time * 20.0) * (cam_rot_y as f32).cos();
                            cam_pos_x += (delta_time * 20.0) * (cam_rot_y as f32).sin();
                        }

                        VirtualKeyCode::Space => {
                            cam_pos_y -= delta_time * 20.0;
                        }
                        VirtualKeyCode::LShift => {
                            cam_pos_y += delta_time * 20.0;
                        }

                        VirtualKeyCode::Right => {
                            cam_rot_y += delta_time;
                        }
                        VirtualKeyCode::Left => {
                            cam_rot_y -= delta_time;
                        }

                        VirtualKeyCode::Up => {
                            cam_rot_x -= delta_time;
                        }
                        VirtualKeyCode::Down => {
                            cam_rot_x += delta_time;
                        }



                        // default handler:
                        _ => { }
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {

                // == // Optionally access the accumulated mouse movement between
                // == // frames here with `delta.0` and `delta.1`

                *delta = (0.0, 0.0); // reset when done
            }

            // SCENE GRAPHS 
            
            
            // draw scene function
            unsafe fn draw_scene(
                simple_shader: &shader::Shader,
                node: &scene_graph::SceneNode,
                view_projection_matrix: &glm::Mat4,
                transformation_so_far: &glm::Mat4,
            ) {
                

                // Calculate the transformation matrix
                let translation_matrix = glm::translation(&(-node.reference_point)); // Move reference point to the origin
                let rotation_matrix_x = glm::rotation(node.rotation.x, &glm::vec3(1.0, 0.0, 0.0));
                let rotation_matrix_y = glm::rotation(node.rotation.y, &glm::vec3(0.0, 1.0, 0.0));
                let rotation_matrix_z = glm::rotation(node.rotation.z, &glm::vec3(0.0, 0.0, 1.0));
                let translation_matrix_back = glm::translation(&node.reference_point); // Move back to the original position

                let model_matrix = transformation_so_far
                    * glm::translation(&node.position)
                    * translation_matrix
                    * rotation_matrix_x
                    * rotation_matrix_y
                    * rotation_matrix_z
                    * translation_matrix_back
                    ;
                
                
                // check if VAO is valid
                if node.vao_id != 0 {
                    //combine the view projection and model matrix
                    //let mvp_matrix = view_projection_matrix * model_matrix;
                    //gl::UniformMatrix4fv(simple_shader.get_uniform_location("identityM"), 1, gl::FALSE, mvp_matrix.as_ptr());

                    // Calculate the Model View Projection matrix
                    let mvp_matrix = view_projection_matrix * model_matrix;

                    //Task 5 b)
                    // Pass the MVP and model matrix to the shader
                 
                    gl::UniformMatrix4fv(simple_shader.get_uniform_location("MVP"), 1, gl::FALSE, mvp_matrix.as_ptr());
                    gl::UniformMatrix4fv(simple_shader.get_uniform_location("model"), 1, gl::FALSE, model_matrix.as_ptr());


                 
                    // Bind the VAO and draw it
                    gl::BindVertexArray(node.vao_id);
                    gl::DrawElements(gl::TRIANGLES, node.index_count, gl::UNSIGNED_INT, ptr::null());
                }

                // Recurse
                for &child in &node.children {
                    //draw_scene(&*child, view_projection_matrix, transformation_so_far);   
                    draw_scene(simple_shader, &*child, view_projection_matrix, &model_matrix);
            
                }
            }



            // == // Please compute camera transforms here (exercise 2 & 3)
            let identity_matrix = glm::Mat4:: identity();

            
            let perspective: glm::Mat4 = glm::perspective(
                window_aspect_ratio,
                70.0f32.to_radians(),
                1.0,
                1000.0,
            );

            let mut matrix = glm::Mat4:: identity();
            matrix = glm::translation(&glm::vec3(0.0, 0.0, -2.0)) * matrix;
            matrix = glm::translation(&glm::vec3(cam_pos_x,cam_pos_y,cam_pos_z)) * matrix;
            matrix = glm::rotation(cam_rot_y, &glm::vec3(0.0, 1.0, 0.0)) * matrix;
            matrix = glm::rotation(cam_rot_x, &glm::vec3(1.0, 0.0, 0.0)) * matrix;
            matrix = perspective * matrix;

            
            //animated path
            for j in 0..5{
                let toolbox: Heading = toolbox::simple_heading_animation(elapsed + (j as f32 * 0.7));

                helicopters[j].position.z = toolbox.z;
                helicopters[j].rotation.x = toolbox.pitch;
                helicopters[j].rotation.z = toolbox.roll;
                helicopters[j].rotation.y = toolbox.yaw;
                helicopters[j].position.x = toolbox.x;

                //Spin the mainrotor and tail
                helicopters[j].get_child(1).rotation.y = elapsed*9.0;
                helicopters[j].get_child(2).rotation.y = elapsed*9.0;

            }

            unsafe {
                // Clear the color and depth buffers
                gl::ClearColor(0.035, 0.046, 0.078, 1.0); // night sky, full opacity
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                draw_scene(&simple_shader, &root_node, &matrix, &identity_matrix);
                
                
                gl::UniformMatrix4fv(simple_shader.get_uniform_location("identityM"), 1, gl::FALSE, matrix.as_ptr());
                /*
            
                // == // Issue the necessary gl:: commands to draw your scene here

                // bind and draw the terrain 
                gl::BindVertexArray(my_vao);
                gl::DrawElements(gl::TRIANGLES, mesh.indices.len() as i32, gl::UNSIGNED_INT, ptr::null());

                //bind and draw the parts of the helicopter Task 2 a
                gl::BindVertexArray(vao_body);
                gl::DrawElements(gl::TRIANGLES, helicopter.body.indices.len() as i32, gl::UNSIGNED_INT, ptr::null());

                gl::BindVertexArray(vao_door);
                gl::DrawElements(gl::TRIANGLES, helicopter.door.indices.len() as i32, gl::UNSIGNED_INT, ptr::null());

                gl::BindVertexArray(vao_main_rotor);
                gl::DrawElements(gl::TRIANGLES, helicopter.main_rotor.indices.len() as i32, gl::UNSIGNED_INT, ptr::null());

                gl::BindVertexArray(vao_tail_rotor);
                gl::DrawElements(gl::TRIANGLES, helicopter.tail_rotor.indices.len() as i32, gl::UNSIGNED_INT, ptr::null());

                 */

            }

            // Display the new color buffer on the display
            context.swap_buffers().unwrap(); // we use "double buffering" to avoid artifacts
        }
    });


    // == //
    // == // From here on down there are only internals.
    // == //


    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events are initially handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::Resized(physical_size), .. } => {
                println!("New window size received: {}x{}", physical_size.width, physical_size.height);
                if let Ok(mut new_size) = arc_window_size.lock() {
                    *new_size = (physical_size.width, physical_size.height, true);
                }
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                    input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, .. }, .. } => {

                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        },
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle Escape and Q keys separately
                match keycode {
                    Escape => { *control_flow = ControlFlow::Exit; }
                    Q      => { *control_flow = ControlFlow::Exit; }
                    _      => { }
                }
            }
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            }
            _ => { }
        }
    });
}
