use include_flate::flate;

flate!(pub static DEFAULT_RAW_X32: [u8] from "data/img/default_x32.png");
flate!(pub static DEFAULT_RAW_X16: [u8] from "data/img/default_x16.png");

flate!(pub static DEFAULT_WARN_RAW_X32: [u8] from "data/img/default_warn_x32.png");
flate!(pub static DEFAULT_WARN_RAW_X16: [u8] from "data/img/default_warn_x16.png");

flate!(pub static DEFAULT_ERROR_RAW_X32: [u8] from "data/img/default_error_x32.png");
flate!(pub static DEFAULT_ERROR_RAW_X16: [u8] from "data/img/default_error_x16.png");
