# RWT
## Rust-Weather-Terminal


A simple app for getting the weather from weather.gov designed for the uConsole R01

This should compile for any platform with ``` cargo ```

run ```rwt``` or ```rwt [zip code]```to launch with a zip from stdin


# Version 0.5.0
## Features
	*Set location from ZIP code
	*Display Forecast information in simple and detaild views
	*Display Alert information from provided ZIP code
	


# Build and Install
## Build

Requires ```curl```

Run ```cargo build -r```	to build a release verson of rwt

## Install
Its recomended that you move the rwt executable file and the data folder to your local bin or any place you like to run terminal applications

## Important
The data folder needs to either be in your working directory or located next to the rwt executable file 