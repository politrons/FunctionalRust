# The Matrix AI - WASM + Rust + Google Gemini API

Welcome to **The Matrix AI** project! This repository contains a Rust-based web server that interacts with Google's Gemini AI API to generate responses to user questions. The server is embedded in a web page using WebAssembly (WASM) to provide a seamless experience

## Project Overview

This project leverages the power of Rust, WebAssembly (WASM), and Google's Gemini AI API to create an interactive web experience. Users can input a question and an API token, which the server uses to query the Gemini AI API and return a generated response.

## Features

- **Rust-based Backend**: The backend is written in Rust and serves as a bridge between the web client and the Gemini AI API.
- **WASM Integration**: The Rust code is compiled to WebAssembly, allowing it to run directly in the browser.
- **Interactive Web Interface**: Users can input questions and their API token through a user-friendly web interface styled like "The Matrix" movie.
- **Real-time AI Responses**: The system queries the Gemini AI API in real-time and displays the responses directly on the web page.

## Requirements

Before you start, ensure you have the following installed:

- **Rust**: The latest version of Rust, including the WASM target.
- **Wasm-Pack**: A tool for building Rust-generated WebAssembly packages.

## Try yourself

You can test how this service works hosted in my Github pages [Here](https://politrons.github.io/FunctionalRust/)

In order to use the service, you will have to obtain a AIStudio API Key [Here](https://aistudio.google.com/app/apikey) 



