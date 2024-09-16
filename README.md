# Rust Bare Metal STM32 Project

This repository contains several Rust-based practices and applications, with a primary focus on embedded programming for an STM32 microcontroller. The project is structured across multiple directories, each demonstrating various concepts in Rust, including recursion, memory management, and hardware control using a real-time, no-OS (bare metal) system.

## Project Structure

### 1. `fibo`
The `fibo` directory contains an implementation of a Fibonacci sequence calculator in Rust. This exercise demonstrates fundamental Rust programming concepts, such as recursion and basic arithmetic, working as a simple exercise.

### 2. `problems`
In this directory, I did some exercises focused on Rust's lifetime and memory management mechanisms. These exercises are crucial for mastering Rust’s unique ownership model and its memory safety guarantees, which are especially important in embedded systems where dynamic memory is scarce or non-existent. And shows the power of RUST in this context.

### 3. `tp-led-matrix`
This directory contains the core of the project—a real-time, bare metal application designed to run on an STM32 microcontroller, specifically controlling a LED matrix. The application interfaces with various peripherals, including a UART for communication, and uses direct register manipulation to interact with the hardware at a low level. This project showcases:
- Real-time control of hardware using Rust.
- Integration of UART communication for sending/receiving data.
- Use of Rust’s `embedded-hal` crate to provide a hardware abstraction layer (HAL) for safer, more maintainable code.
- LED matrix control through efficient bit-level manipulation.

## Bare Metal Implementation

The bare metal STM32 application is built using several libraries that facilitate low-level hardware control and communication, all while leveraging Rust’s safety features. The core components of the application include:

- **Real-time bare metal programming**: The application is designed to run without an operating system, directly interacting with the STM32 hardware through memory-mapped registers.
- **UART communication**: UART is used for bidirectional communication with the microcontroller, facilitating debugging, control, or interaction with other devices.
- **LED matrix control**: The application implements direct control of a connected LED matrix, using GPIO and possibly SPI communication for efficient data transfer.
- **Gamma correction and image processing**: Advanced techniques, such as gamma correction and efficient image rendering, are employed to ensure the LED matrix displays visually appealing outputs.
- **Hardware Abstraction Layer (HAL)**: The Rust `embedded-hal` crate is used to abstract away the hardware specifics, providing a cleaner and safer interface for interacting with peripherals like GPIO, timers, and communication interfaces (SPI/UART).
- **Embassy Framework**: This project also integrates the `Embassy` asynchronous embedded framework for Rust. Embassy provides non-blocking, cooperative multitasking capabilities which are crucial for real-time embedded systems. The project leverages Embassy’s abstractions for handling asynchronous tasks such as UART communication and LED matrix updates without the need for a real-time operating system (RTOS).

Overall, this project demonstrates the capabilities of Rust in bare metal programming, leveraging its safety and concurrency features while maintaining low-level control necessary for embedded systems. 
It is even more interesting to compare with the same application seen in : [GitHub Repository: Bare Metal STM32](https://github.com/guduart02sa/bare_metal_STM32), which applies all the same using C.
