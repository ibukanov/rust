// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/*!
 * Low-level bindings to the libuv library.
 *
 * This module contains a set of direct, 'bare-metal' wrappers around
 * the libuv C-API.
 *
 * We're not bothering yet to redefine uv's structs as Rust structs
 * because they are quite large and change often between versions.
 * The maintenance burden is just too high. Instead we use the uv's
 * `uv_handle_size` and `uv_req_size` to find the correct size of the
 * structs and allocate them on the heap. This can be revisited later.
 *
 * There are also a collection of helper functions to ease interacting
 * with the low-level API.
 *
 * As new functionality, existent in uv.h, is added to the rust stdlib,
 * the mappings should be added in this module.
 */

#[allow(non_camel_case_types)]; // C types

use c_str::ToCStr;
use libc::{size_t, c_int, c_uint, c_void, c_char, uintptr_t};
use libc::ssize_t;
use libc::{malloc, free};
use libc;
use prelude::*;
use ptr;
use vec;

pub use self::errors::*;

pub static OK: c_int = 0;
pub static EOF: c_int = -4095;
pub static UNKNOWN: c_int = -4094;

// uv-errno.h redefines error codes for windows, but not for unix...

#[cfg(windows)]
pub mod errors {
    use libc::c_int;

    pub static EACCES: c_int = -4093;
    pub static ECONNREFUSED: c_int = -4079;
    pub static ECONNRESET: c_int = -4078;
    pub static ENOTCONN: c_int = -4054;
    pub static EPIPE: c_int = -4048;
}
#[cfg(not(windows))]
pub mod errors {
    use libc;
    use libc::c_int;

    pub static EACCES: c_int = -libc::EACCES;
    pub static ECONNREFUSED: c_int = -libc::ECONNREFUSED;
    pub static ECONNRESET: c_int = -libc::ECONNRESET;
    pub static ENOTCONN: c_int = -libc::ENOTCONN;
    pub static EPIPE: c_int = -libc::EPIPE;
}

pub static PROCESS_SETUID: c_int = 1 << 0;
pub static PROCESS_SETGID: c_int = 1 << 1;
pub static PROCESS_WINDOWS_VERBATIM_ARGUMENTS: c_int = 1 << 2;
pub static PROCESS_DETACHED: c_int = 1 << 3;
pub static PROCESS_WINDOWS_HIDE: c_int = 1 << 4;

pub static STDIO_IGNORE: c_int = 0x00;
pub static STDIO_CREATE_PIPE: c_int = 0x01;
pub static STDIO_INHERIT_FD: c_int = 0x02;
pub static STDIO_INHERIT_STREAM: c_int = 0x04;
pub static STDIO_READABLE_PIPE: c_int = 0x10;
pub static STDIO_WRITABLE_PIPE: c_int = 0x20;

// see libuv/include/uv-unix.h
#[cfg(unix)]
pub struct uv_buf_t {
    base: *u8,
    len: libc::size_t,
}

// see libuv/include/uv-win.h
#[cfg(windows)]
pub struct uv_buf_t {
    len: u32,
    base: *u8,
}

pub struct uv_process_options_t {
    exit_cb: uv_exit_cb,
    file: *libc::c_char,
    args: **libc::c_char,
    env: **libc::c_char,
    cwd: *libc::c_char,
    flags: libc::c_uint,
    stdio_count: libc::c_int,
    stdio: *uv_stdio_container_t,
    uid: uv_uid_t,
    gid: uv_gid_t,
}

// These fields are private because they must be interfaced with through the
// functions below.
pub struct uv_stdio_container_t {
    priv flags: libc::c_int,
    priv stream: *uv_stream_t,
}

pub type uv_handle_t = c_void;
pub type uv_loop_t = c_void;
pub type uv_idle_t = c_void;
pub type uv_tcp_t = c_void;
pub type uv_udp_t = c_void;
pub type uv_connect_t = c_void;
pub type uv_connection_t = c_void;
pub type uv_write_t = c_void;
pub type uv_async_t = c_void;
pub type uv_timer_t = c_void;
pub type uv_stream_t = c_void;
pub type uv_fs_t = c_void;
pub type uv_udp_send_t = c_void;
pub type uv_getaddrinfo_t = c_void;
pub type uv_process_t = c_void;
pub type uv_pipe_t = c_void;

pub struct uv_timespec_t {
    tv_sec: libc::c_long,
    priv tv_nsec: libc::c_long
}

pub struct uv_stat_t {
    st_dev: libc::uint64_t,
    st_mode: libc::uint64_t,
    priv st_nlink: libc::uint64_t,
    priv st_uid: libc::uint64_t,
    priv st_gid: libc::uint64_t,
    priv st_rdev: libc::uint64_t,
    st_ino: libc::uint64_t,
    st_size: libc::uint64_t,
    priv st_blksize: libc::uint64_t,
    priv st_blocks: libc::uint64_t,
    priv st_flags: libc::uint64_t,
    priv st_gen: libc::uint64_t,
    st_atim: uv_timespec_t,
    st_mtim: uv_timespec_t,
    st_ctim: uv_timespec_t,
    priv st_birthtim: uv_timespec_t
}

impl uv_stat_t {
    pub fn new() -> uv_stat_t {
        uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t { tv_sec: 0, tv_nsec: 0 },
            st_mtim: uv_timespec_t { tv_sec: 0, tv_nsec: 0 },
            st_ctim: uv_timespec_t { tv_sec: 0, tv_nsec: 0 },
            st_birthtim: uv_timespec_t { tv_sec: 0, tv_nsec: 0 }
        }
    }
    pub fn is_file(&self) -> bool {
        ((self.st_mode) & libc::S_IFMT as libc::uint64_t) == libc::S_IFREG as libc::uint64_t
    }
    pub fn is_dir(&self) -> bool {
        ((self.st_mode) & libc::S_IFMT as libc::uint64_t) == libc::S_IFDIR as libc::uint64_t
    }
}

pub type uv_idle_cb = extern "C" fn(handle: *uv_idle_t,
                                    status: c_int);
pub type uv_alloc_cb = extern "C" fn(stream: *uv_stream_t,
                                     suggested_size: size_t) -> uv_buf_t;
pub type uv_read_cb = extern "C" fn(stream: *uv_stream_t,
                                    nread: ssize_t,
                                    buf: uv_buf_t);
pub type uv_udp_send_cb = extern "C" fn(req: *uv_udp_send_t,
                                        status: c_int);
pub type uv_udp_recv_cb = extern "C" fn(handle: *uv_udp_t,
                                        nread: ssize_t,
                                        buf: uv_buf_t,
                                        addr: *sockaddr,
                                        flags: c_uint);
pub type uv_close_cb = extern "C" fn(handle: *uv_handle_t);
pub type uv_walk_cb = extern "C" fn(handle: *uv_handle_t,
                                    arg: *c_void);
pub type uv_async_cb = extern "C" fn(handle: *uv_async_t,
                                     status: c_int);
pub type uv_connect_cb = extern "C" fn(handle: *uv_connect_t,
                                       status: c_int);
pub type uv_connection_cb = extern "C" fn(handle: *uv_connection_t,
                                          status: c_int);
pub type uv_timer_cb = extern "C" fn(handle: *uv_timer_t,
                                     status: c_int);
pub type uv_write_cb = extern "C" fn(handle: *uv_write_t,
                                     status: c_int);
pub type uv_getaddrinfo_cb = extern "C" fn(req: *uv_getaddrinfo_t,
                                           status: c_int,
                                           res: *addrinfo);
pub type uv_exit_cb = extern "C" fn(handle: *uv_process_t,
                                    exit_status: c_int,
                                    term_signal: c_int);

pub type sockaddr = c_void;
pub type sockaddr_in = c_void;
pub type sockaddr_in6 = c_void;
pub type sockaddr_storage = c_void;

#[cfg(unix)]
pub type socklen_t = c_int;

// XXX: This is a standard C type. Could probably be defined in libc
#[cfg(target_os = "android")]
#[cfg(target_os = "linux")]
pub struct addrinfo {
    priv ai_flags: c_int,
    priv ai_family: c_int,
    priv ai_socktype: c_int,
    priv ai_protocol: c_int,
    priv ai_addrlen: socklen_t,
    ai_addr: *sockaddr,
    priv ai_canonname: *char,
    ai_next: *addrinfo
}

#[cfg(target_os = "macos")]
#[cfg(target_os = "freebsd")]
pub struct addrinfo {
    priv ai_flags: c_int,
    priv ai_family: c_int,
    priv ai_socktype: c_int,
    priv ai_protocol: c_int,
    priv ai_addrlen: socklen_t,
    priv ai_canonname: *char,
    ai_addr: *sockaddr,
    ai_next: *addrinfo
}

#[cfg(windows)]
pub struct addrinfo {
    priv ai_flags: c_int,
    priv ai_family: c_int,
    priv ai_socktype: c_int,
    priv ai_protocol: c_int,
    priv ai_addrlen: size_t,
    priv ai_canonname: *char,
    ai_addr: *sockaddr,
    ai_next: *addrinfo
}

#[cfg(unix)] pub type uv_uid_t = libc::types::os::arch::posix88::uid_t;
#[cfg(unix)] pub type uv_gid_t = libc::types::os::arch::posix88::gid_t;
#[cfg(windows)] pub type uv_uid_t = libc::c_uchar;
#[cfg(windows)] pub type uv_gid_t = libc::c_uchar;

#[deriving(Eq)]
pub enum uv_handle_type {
    UV_UNKNOWN_HANDLE,
    UV_ASYNC,
    UV_CHECK,
    UV_FS_EVENT,
    UV_FS_POLL,
    UV_HANDLE,
    UV_IDLE,
    UV_NAMED_PIPE,
    UV_POLL,
    UV_PREPARE,
    UV_PROCESS,
    UV_STREAM,
    UV_TCP,
    UV_TIMER,
    UV_TTY,
    UV_UDP,
    UV_SIGNAL,
    UV_FILE,
    UV_HANDLE_TYPE_MAX
}

#[cfg(unix)]
#[deriving(Eq)]
pub enum uv_req_type {
    UV_UNKNOWN_REQ,
    UV_REQ,
    UV_CONNECT,
    UV_WRITE,
    UV_SHUTDOWN,
    UV_UDP_SEND,
    UV_FS,
    UV_WORK,
    UV_GETADDRINFO,
    UV_REQ_TYPE_MAX
}

// uv_req_type may have additional fields defined by UV_REQ_TYPE_PRIVATE.
// See UV_REQ_TYPE_PRIVATE at libuv/include/uv-win.h
#[cfg(windows)]
#[deriving(Eq)]
pub enum uv_req_type {
    UV_UNKNOWN_REQ,
    UV_REQ,
    UV_CONNECT,
    UV_WRITE,
    UV_SHUTDOWN,
    UV_UDP_SEND,
    UV_FS,
    UV_WORK,
    UV_GETADDRINFO,
    UV_ACCEPT,
    UV_FS_EVENT_REQ,
    UV_POLL_REQ,
    UV_PROCESS_EXIT,
    UV_READ,
    UV_UDP_RECV,
    UV_WAKEUP,
    UV_SIGNAL_REQ,
    UV_REQ_TYPE_MAX
}

#[deriving(Eq)]
pub enum uv_membership {
    UV_LEAVE_GROUP,
    UV_JOIN_GROUP
}

pub unsafe fn malloc_handle(handle: uv_handle_type) -> *c_void {
    #[fixed_stack_segment]; #[inline(never)];

    assert!(handle != UV_UNKNOWN_HANDLE && handle != UV_HANDLE_TYPE_MAX);
    let size = rust_uv_handle_size(handle as uint);
    let p = malloc(size);
    assert!(p.is_not_null());
    return p;
}

pub unsafe fn free_handle(v: *c_void) {
    #[fixed_stack_segment]; #[inline(never)];

    free(v)
}

pub unsafe fn malloc_req(req: uv_req_type) -> *c_void {
    #[fixed_stack_segment]; #[inline(never)];

    assert!(req != UV_UNKNOWN_REQ && req != UV_REQ_TYPE_MAX);
    let size = rust_uv_req_size(req as uint);
    let p = malloc(size);
    assert!(p.is_not_null());
    return p;
}

pub unsafe fn free_req(v: *c_void) {
    #[fixed_stack_segment]; #[inline(never)];

    free(v)
}

#[test]
fn handle_sanity_check() {
    #[fixed_stack_segment]; #[inline(never)];
    unsafe {
        assert_eq!(UV_HANDLE_TYPE_MAX as uint, rust_uv_handle_type_max());
    }
}

#[test]
fn request_sanity_check() {
    #[fixed_stack_segment]; #[inline(never)];
    unsafe {
        assert_eq!(UV_REQ_TYPE_MAX as uint, rust_uv_req_type_max());
    }
}

// XXX Event loops ignore SIGPIPE by default.
pub unsafe fn loop_new() -> *c_void {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_loop_new();
}

pub unsafe fn loop_delete(loop_handle: *c_void) {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_loop_delete(loop_handle);
}

pub unsafe fn run(loop_handle: *c_void) {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_run(loop_handle);
}

pub unsafe fn close<T>(handle: *T, cb: uv_close_cb) {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_close(handle as *c_void, cb);
}

pub unsafe fn walk(loop_handle: *c_void, cb: uv_walk_cb, arg: *c_void) {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_walk(loop_handle, cb, arg);
}

pub unsafe fn idle_new() -> *uv_idle_t {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_idle_new()
}

pub unsafe fn idle_delete(handle: *uv_idle_t) {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_idle_delete(handle)
}

pub unsafe fn idle_init(loop_handle: *uv_loop_t, handle: *uv_idle_t) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_idle_init(loop_handle, handle)
}

pub unsafe fn idle_start(handle: *uv_idle_t, cb: uv_idle_cb) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_idle_start(handle, cb)
}

pub unsafe fn idle_stop(handle: *uv_idle_t) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_idle_stop(handle)
}

pub unsafe fn udp_init(loop_handle: *uv_loop_t, handle: *uv_udp_t) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_udp_init(loop_handle, handle);
}

pub unsafe fn udp_bind(server: *uv_udp_t, addr: *sockaddr_in, flags: c_uint) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_udp_bind(server, addr, flags);
}

pub unsafe fn udp_bind6(server: *uv_udp_t, addr: *sockaddr_in6, flags: c_uint) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_udp_bind6(server, addr, flags);
}

pub unsafe fn udp_send<T>(req: *uv_udp_send_t, handle: *T, buf_in: &[uv_buf_t],
                          addr: *sockaddr_in, cb: uv_udp_send_cb) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    let buf_ptr = vec::raw::to_ptr(buf_in);
    let buf_cnt = buf_in.len() as i32;
    return rust_uv_udp_send(req, handle as *c_void, buf_ptr, buf_cnt, addr, cb);
}

pub unsafe fn udp_send6<T>(req: *uv_udp_send_t, handle: *T, buf_in: &[uv_buf_t],
                          addr: *sockaddr_in6, cb: uv_udp_send_cb) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    let buf_ptr = vec::raw::to_ptr(buf_in);
    let buf_cnt = buf_in.len() as i32;
    return rust_uv_udp_send6(req, handle as *c_void, buf_ptr, buf_cnt, addr, cb);
}

pub unsafe fn udp_recv_start(server: *uv_udp_t, on_alloc: uv_alloc_cb,
                             on_recv: uv_udp_recv_cb) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_udp_recv_start(server, on_alloc, on_recv);
}

pub unsafe fn udp_recv_stop(server: *uv_udp_t) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_udp_recv_stop(server);
}

pub unsafe fn get_udp_handle_from_send_req(send_req: *uv_udp_send_t) -> *uv_udp_t {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_get_udp_handle_from_send_req(send_req);
}

pub unsafe fn udp_getsockname(handle: *uv_udp_t, name: *sockaddr_storage) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_udp_getsockname(handle, name);
}

pub unsafe fn udp_set_membership(handle: *uv_udp_t, multicast_addr: *c_char,
                                 interface_addr: *c_char, membership: uv_membership) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_udp_set_membership(handle, multicast_addr, interface_addr, membership as c_int);
}

pub unsafe fn udp_set_multicast_loop(handle: *uv_udp_t, on: c_int) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_udp_set_multicast_loop(handle, on);
}

pub unsafe fn udp_set_multicast_ttl(handle: *uv_udp_t, ttl: c_int) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_udp_set_multicast_ttl(handle, ttl);
}

pub unsafe fn udp_set_ttl(handle: *uv_udp_t, ttl: c_int) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_udp_set_ttl(handle, ttl);
}

pub unsafe fn udp_set_broadcast(handle: *uv_udp_t, on: c_int) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_udp_set_broadcast(handle, on);
}

pub unsafe fn tcp_init(loop_handle: *c_void, handle: *uv_tcp_t) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_tcp_init(loop_handle, handle);
}

pub unsafe fn tcp_connect(connect_ptr: *uv_connect_t, tcp_handle_ptr: *uv_tcp_t,
                          addr_ptr: *sockaddr_in, after_connect_cb: uv_connect_cb) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_tcp_connect(connect_ptr, tcp_handle_ptr, after_connect_cb, addr_ptr);
}

pub unsafe fn tcp_connect6(connect_ptr: *uv_connect_t, tcp_handle_ptr: *uv_tcp_t,
                           addr_ptr: *sockaddr_in6, after_connect_cb: uv_connect_cb) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_tcp_connect6(connect_ptr, tcp_handle_ptr, after_connect_cb, addr_ptr);
}

pub unsafe fn tcp_bind(tcp_server_ptr: *uv_tcp_t, addr_ptr: *sockaddr_in) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_tcp_bind(tcp_server_ptr, addr_ptr);
}

pub unsafe fn tcp_bind6(tcp_server_ptr: *uv_tcp_t, addr_ptr: *sockaddr_in6) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_tcp_bind6(tcp_server_ptr, addr_ptr);
}

pub unsafe fn tcp_getpeername(tcp_handle_ptr: *uv_tcp_t, name: *sockaddr_storage) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_tcp_getpeername(tcp_handle_ptr, name);
}

pub unsafe fn tcp_getsockname(handle: *uv_tcp_t, name: *sockaddr_storage) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_tcp_getsockname(handle, name);
}

pub unsafe fn tcp_nodelay(handle: *uv_tcp_t, enable: c_int) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_tcp_nodelay(handle, enable);
}

pub unsafe fn tcp_keepalive(handle: *uv_tcp_t, enable: c_int, delay: c_uint) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_tcp_keepalive(handle, enable, delay);
}

pub unsafe fn tcp_simultaneous_accepts(handle: *uv_tcp_t, enable: c_int) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_tcp_simultaneous_accepts(handle, enable);
}

pub unsafe fn listen<T>(stream: *T, backlog: c_int,
                        cb: uv_connection_cb) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_listen(stream as *c_void, backlog, cb);
}

pub unsafe fn accept(server: *c_void, client: *c_void) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_accept(server as *c_void, client as *c_void);
}

pub unsafe fn write<T>(req: *uv_write_t,
                       stream: *T,
                       buf_in: &[uv_buf_t],
                       cb: uv_write_cb) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    let buf_ptr = vec::raw::to_ptr(buf_in);
    let buf_cnt = buf_in.len() as i32;
    return rust_uv_write(req as *c_void, stream as *c_void, buf_ptr, buf_cnt, cb);
}
pub unsafe fn read_start(stream: *uv_stream_t,
                         on_alloc: uv_alloc_cb,
                         on_read: uv_read_cb) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_read_start(stream as *c_void, on_alloc, on_read);
}

pub unsafe fn read_stop(stream: *uv_stream_t) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_read_stop(stream as *c_void);
}

pub unsafe fn strerror(err: c_int) -> *c_char {
    #[fixed_stack_segment]; #[inline(never)];
    return rust_uv_strerror(err);
}
pub unsafe fn err_name(err: c_int) -> *c_char {
    #[fixed_stack_segment]; #[inline(never)];
    return rust_uv_err_name(err);
}

pub unsafe fn async_init(loop_handle: *c_void,
                         async_handle: *uv_async_t,
                         cb: uv_async_cb) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_async_init(loop_handle, async_handle, cb);
}

pub unsafe fn async_send(async_handle: *uv_async_t) {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_async_send(async_handle);
}
pub unsafe fn buf_init(input: *u8, len: uint) -> uv_buf_t {
    #[fixed_stack_segment]; #[inline(never)];

    let out_buf = uv_buf_t { base: ptr::null(), len: 0 as size_t };
    let out_buf_ptr = ptr::to_unsafe_ptr(&out_buf);
    rust_uv_buf_init(out_buf_ptr, input, len as size_t);
    return out_buf;
}

pub unsafe fn timer_init(loop_ptr: *c_void, timer_ptr: *uv_timer_t) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_timer_init(loop_ptr, timer_ptr);
}
pub unsafe fn timer_start(timer_ptr: *uv_timer_t,
                          cb: uv_timer_cb, timeout: u64,
                          repeat: u64) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_timer_start(timer_ptr, cb, timeout, repeat);
}
pub unsafe fn timer_stop(timer_ptr: *uv_timer_t) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_timer_stop(timer_ptr);
}

pub unsafe fn is_ip4_addr(addr: *sockaddr) -> bool {
    #[fixed_stack_segment]; #[inline(never)];

    match rust_uv_is_ipv4_sockaddr(addr) { 0 => false, _ => true }
}

pub unsafe fn is_ip6_addr(addr: *sockaddr) -> bool {
    #[fixed_stack_segment]; #[inline(never)];

    match rust_uv_is_ipv6_sockaddr(addr) { 0 => false, _ => true }
}

pub unsafe fn malloc_ip4_addr(ip: &str, port: int) -> *sockaddr_in {
    #[fixed_stack_segment]; #[inline(never)];
    do ip.with_c_str |ip_buf| {
        rust_uv_ip4_addrp(ip_buf as *u8, port as libc::c_int)
    }
}
pub unsafe fn malloc_ip6_addr(ip: &str, port: int) -> *sockaddr_in6 {
    #[fixed_stack_segment]; #[inline(never)];
    do ip.with_c_str |ip_buf| {
        rust_uv_ip6_addrp(ip_buf as *u8, port as libc::c_int)
    }
}

pub unsafe fn malloc_sockaddr_storage() -> *sockaddr_storage {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_malloc_sockaddr_storage()
}

pub unsafe fn free_sockaddr_storage(ss: *sockaddr_storage) {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_free_sockaddr_storage(ss);
}

pub unsafe fn free_ip4_addr(addr: *sockaddr_in) {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_free_ip4_addr(addr);
}

pub unsafe fn free_ip6_addr(addr: *sockaddr_in6) {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_free_ip6_addr(addr);
}

pub unsafe fn ip4_name(addr: *sockaddr_in, dst: *u8, size: size_t) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_ip4_name(addr, dst, size);
}

pub unsafe fn ip6_name(addr: *sockaddr_in6, dst: *u8, size: size_t) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_ip6_name(addr, dst, size);
}

pub unsafe fn ip4_port(addr: *sockaddr_in) -> c_uint {
    #[fixed_stack_segment]; #[inline(never)];

   return rust_uv_ip4_port(addr);
}

pub unsafe fn ip6_port(addr: *sockaddr_in6) -> c_uint {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_ip6_port(addr);
}

pub unsafe fn fs_open(loop_ptr: *uv_loop_t, req: *uv_fs_t, path: *c_char, flags: int, mode: int,
                cb: *u8) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_fs_open(loop_ptr, req, path, flags as c_int, mode as c_int, cb)
}

pub unsafe fn fs_unlink(loop_ptr: *uv_loop_t, req: *uv_fs_t, path: *c_char,
                cb: *u8) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_fs_unlink(loop_ptr, req, path, cb)
}
pub unsafe fn fs_write(loop_ptr: *uv_loop_t, req: *uv_fs_t, fd: c_int, buf: *c_void,
                       len: uint, offset: i64, cb: *u8) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_fs_write(loop_ptr, req, fd, buf, len as c_uint, offset, cb)
}
pub unsafe fn fs_read(loop_ptr: *uv_loop_t, req: *uv_fs_t, fd: c_int, buf: *c_void,
                       len: uint, offset: i64, cb: *u8) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_fs_read(loop_ptr, req, fd, buf, len as c_uint, offset, cb)
}
pub unsafe fn fs_close(loop_ptr: *uv_loop_t, req: *uv_fs_t, fd: c_int,
                cb: *u8) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_fs_close(loop_ptr, req, fd, cb)
}
pub unsafe fn fs_stat(loop_ptr: *uv_loop_t, req: *uv_fs_t, path: *c_char, cb: *u8) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_fs_stat(loop_ptr, req, path, cb)
}
pub unsafe fn fs_fstat(loop_ptr: *uv_loop_t, req: *uv_fs_t, fd: c_int, cb: *u8) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_fs_fstat(loop_ptr, req, fd, cb)
}
pub unsafe fn fs_mkdir(loop_ptr: *uv_loop_t, req: *uv_fs_t, path: *c_char, mode: int,
                cb: *u8) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_fs_mkdir(loop_ptr, req, path, mode as c_int, cb)
}
pub unsafe fn fs_rmdir(loop_ptr: *uv_loop_t, req: *uv_fs_t, path: *c_char,
                cb: *u8) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_fs_rmdir(loop_ptr, req, path, cb)
}
pub unsafe fn fs_readdir(loop_ptr: *uv_loop_t, req: *uv_fs_t, path: *c_char,
                flags: c_int, cb: *u8) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_fs_readdir(loop_ptr, req, path, flags, cb)
}
pub unsafe fn populate_stat(req_in: *uv_fs_t, stat_out: *uv_stat_t) {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_populate_uv_stat(req_in, stat_out)
}
pub unsafe fn fs_req_cleanup(req: *uv_fs_t) {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_fs_req_cleanup(req);
}

pub unsafe fn spawn(loop_ptr: *c_void, result: *uv_process_t,
                    options: uv_process_options_t) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];
    return rust_uv_spawn(loop_ptr, result, options);
}

pub unsafe fn process_kill(p: *uv_process_t, signum: c_int) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];
    return rust_uv_process_kill(p, signum);
}

pub unsafe fn process_pid(p: *uv_process_t) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];
    return rust_uv_process_pid(p);
}

pub unsafe fn set_stdio_container_flags(c: *uv_stdio_container_t,
                                        flags: libc::c_int) {
    #[fixed_stack_segment]; #[inline(never)];
    rust_set_stdio_container_flags(c, flags);
}

pub unsafe fn set_stdio_container_fd(c: *uv_stdio_container_t,
                                     fd: libc::c_int) {
    #[fixed_stack_segment]; #[inline(never)];
    rust_set_stdio_container_fd(c, fd);
}

pub unsafe fn set_stdio_container_stream(c: *uv_stdio_container_t,
                                         stream: *uv_stream_t) {
    #[fixed_stack_segment]; #[inline(never)];
    rust_set_stdio_container_stream(c, stream);
}

pub unsafe fn pipe_init(loop_ptr: *c_void, p: *uv_pipe_t, ipc: c_int) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];
    rust_uv_pipe_init(loop_ptr, p, ipc)
}

// data access helpers
pub unsafe fn get_result_from_fs_req(req: *uv_fs_t) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_get_result_from_fs_req(req)
}
pub unsafe fn get_ptr_from_fs_req(req: *uv_fs_t) -> *libc::c_void {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_get_ptr_from_fs_req(req)
}
pub unsafe fn get_loop_from_fs_req(req: *uv_fs_t) -> *uv_loop_t {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_get_loop_from_fs_req(req)
}
pub unsafe fn get_loop_from_getaddrinfo_req(req: *uv_getaddrinfo_t) -> *uv_loop_t {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_get_loop_from_getaddrinfo_req(req)
}
pub unsafe fn get_loop_for_uv_handle<T>(handle: *T) -> *c_void {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_get_loop_for_uv_handle(handle as *c_void);
}
pub unsafe fn get_stream_handle_from_connect_req(connect: *uv_connect_t) -> *uv_stream_t {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_get_stream_handle_from_connect_req(connect);
}
pub unsafe fn get_stream_handle_from_write_req(write_req: *uv_write_t) -> *uv_stream_t {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_get_stream_handle_from_write_req(write_req);
}
pub unsafe fn get_data_for_uv_loop(loop_ptr: *c_void) -> *c_void {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_get_data_for_uv_loop(loop_ptr)
}
pub unsafe fn set_data_for_uv_loop(loop_ptr: *c_void, data: *c_void) {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_set_data_for_uv_loop(loop_ptr, data);
}
pub unsafe fn get_data_for_uv_handle<T>(handle: *T) -> *c_void {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_get_data_for_uv_handle(handle as *c_void);
}
pub unsafe fn set_data_for_uv_handle<T, U>(handle: *T, data: *U) {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_set_data_for_uv_handle(handle as *c_void, data as *c_void);
}
pub unsafe fn get_data_for_req<T>(req: *T) -> *c_void {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_get_data_for_req(req as *c_void);
}
pub unsafe fn set_data_for_req<T, U>(req: *T, data: *U) {
    #[fixed_stack_segment]; #[inline(never)];

    rust_uv_set_data_for_req(req as *c_void, data as *c_void);
}
pub unsafe fn get_base_from_buf(buf: uv_buf_t) -> *u8 {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_get_base_from_buf(buf);
}
pub unsafe fn get_len_from_buf(buf: uv_buf_t) -> size_t {
    #[fixed_stack_segment]; #[inline(never)];

    return rust_uv_get_len_from_buf(buf);
}
pub unsafe fn getaddrinfo(loop_: *uv_loop_t, req: *uv_getaddrinfo_t,
               getaddrinfo_cb: uv_getaddrinfo_cb,
               node: *c_char, service: *c_char,
               hints: *addrinfo) -> c_int {
    #[fixed_stack_segment]; #[inline(never)];
    return rust_uv_getaddrinfo(loop_, req, getaddrinfo_cb, node, service, hints);
}
pub unsafe fn freeaddrinfo(ai: *addrinfo) {
    #[fixed_stack_segment]; #[inline(never)];
    rust_uv_freeaddrinfo(ai);
}

pub struct uv_err_data {
    priv err_name: ~str,
    priv err_msg: ~str,
}

extern {

    fn rust_uv_handle_size(type_: uintptr_t) -> size_t;
    fn rust_uv_req_size(type_: uintptr_t) -> size_t;
    fn rust_uv_handle_type_max() -> uintptr_t;
    fn rust_uv_req_type_max() -> uintptr_t;

    // libuv public API
    fn rust_uv_loop_new() -> *c_void;
    fn rust_uv_loop_delete(lp: *c_void);
    fn rust_uv_run(loop_handle: *c_void);
    fn rust_uv_close(handle: *c_void, cb: uv_close_cb);
    fn rust_uv_walk(loop_handle: *c_void, cb: uv_walk_cb, arg: *c_void);

    fn rust_uv_idle_new() -> *uv_idle_t;
    fn rust_uv_idle_delete(handle: *uv_idle_t);
    fn rust_uv_idle_init(loop_handle: *uv_loop_t, handle: *uv_idle_t) -> c_int;
    fn rust_uv_idle_start(handle: *uv_idle_t, cb: uv_idle_cb) -> c_int;
    fn rust_uv_idle_stop(handle: *uv_idle_t) -> c_int;

    fn rust_uv_async_send(handle: *uv_async_t);
    fn rust_uv_async_init(loop_handle: *c_void,
                          async_handle: *uv_async_t,
                          cb: uv_async_cb) -> c_int;
    fn rust_uv_tcp_init(loop_handle: *c_void, handle_ptr: *uv_tcp_t) -> c_int;
    fn rust_uv_buf_init(out_buf: *uv_buf_t, base: *u8, len: size_t);
    fn rust_uv_strerror(err: c_int) -> *c_char;
    fn rust_uv_err_name(err: c_int) -> *c_char;
    fn rust_uv_ip4_addrp(ip: *u8, port: c_int) -> *sockaddr_in;
    fn rust_uv_ip6_addrp(ip: *u8, port: c_int) -> *sockaddr_in6;
    fn rust_uv_free_ip4_addr(addr: *sockaddr_in);
    fn rust_uv_free_ip6_addr(addr: *sockaddr_in6);
    fn rust_uv_ip4_name(src: *sockaddr_in, dst: *u8, size: size_t) -> c_int;
    fn rust_uv_ip6_name(src: *sockaddr_in6, dst: *u8, size: size_t) -> c_int;
    fn rust_uv_ip4_port(src: *sockaddr_in) -> c_uint;
    fn rust_uv_ip6_port(src: *sockaddr_in6) -> c_uint;
    fn rust_uv_tcp_connect(req: *uv_connect_t, handle: *uv_tcp_t,
                           cb: uv_connect_cb,
                           addr: *sockaddr_in) -> c_int;
    fn rust_uv_tcp_bind(tcp_server: *uv_tcp_t, addr: *sockaddr_in) -> c_int;
    fn rust_uv_tcp_connect6(req: *uv_connect_t, handle: *uv_tcp_t,
                            cb: uv_connect_cb,
                            addr: *sockaddr_in6) -> c_int;
    fn rust_uv_tcp_bind6(tcp_server: *uv_tcp_t, addr: *sockaddr_in6) -> c_int;
    fn rust_uv_tcp_getpeername(tcp_handle_ptr: *uv_tcp_t, name: *sockaddr_storage) -> c_int;
    fn rust_uv_tcp_getsockname(handle: *uv_tcp_t, name: *sockaddr_storage) -> c_int;
    fn rust_uv_tcp_nodelay(handle: *uv_tcp_t, enable: c_int) -> c_int;
    fn rust_uv_tcp_keepalive(handle: *uv_tcp_t, enable: c_int, delay: c_uint) -> c_int;
    fn rust_uv_tcp_simultaneous_accepts(handle: *uv_tcp_t, enable: c_int) -> c_int;

    fn rust_uv_udp_init(loop_handle: *uv_loop_t, handle_ptr: *uv_udp_t) -> c_int;
    fn rust_uv_udp_bind(server: *uv_udp_t, addr: *sockaddr_in, flags: c_uint) -> c_int;
    fn rust_uv_udp_bind6(server: *uv_udp_t, addr: *sockaddr_in6, flags: c_uint) -> c_int;
    fn rust_uv_udp_send(req: *uv_udp_send_t, handle: *uv_udp_t, buf_in: *uv_buf_t,
                        buf_cnt: c_int, addr: *sockaddr_in, cb: uv_udp_send_cb) -> c_int;
    fn rust_uv_udp_send6(req: *uv_udp_send_t, handle: *uv_udp_t, buf_in: *uv_buf_t,
                         buf_cnt: c_int, addr: *sockaddr_in6, cb: uv_udp_send_cb) -> c_int;
    fn rust_uv_udp_recv_start(server: *uv_udp_t,
                              on_alloc: uv_alloc_cb,
                              on_recv: uv_udp_recv_cb) -> c_int;
    fn rust_uv_udp_recv_stop(server: *uv_udp_t) -> c_int;
    fn rust_uv_get_udp_handle_from_send_req(req: *uv_udp_send_t) -> *uv_udp_t;
    fn rust_uv_udp_getsockname(handle: *uv_udp_t, name: *sockaddr_storage) -> c_int;
    fn rust_uv_udp_set_membership(handle: *uv_udp_t, multicast_addr: *c_char,
                                  interface_addr: *c_char, membership: c_int) -> c_int;
    fn rust_uv_udp_set_multicast_loop(handle: *uv_udp_t, on: c_int) -> c_int;
    fn rust_uv_udp_set_multicast_ttl(handle: *uv_udp_t, ttl: c_int) -> c_int;
    fn rust_uv_udp_set_ttl(handle: *uv_udp_t, ttl: c_int) -> c_int;
    fn rust_uv_udp_set_broadcast(handle: *uv_udp_t, on: c_int) -> c_int;

    fn rust_uv_is_ipv4_sockaddr(addr: *sockaddr) -> c_int;
    fn rust_uv_is_ipv6_sockaddr(addr: *sockaddr) -> c_int;
    fn rust_uv_malloc_sockaddr_storage() -> *sockaddr_storage;
    fn rust_uv_free_sockaddr_storage(ss: *sockaddr_storage);

    fn rust_uv_listen(stream: *c_void, backlog: c_int,
                      cb: uv_connection_cb) -> c_int;
    fn rust_uv_accept(server: *c_void, client: *c_void) -> c_int;
    fn rust_uv_write(req: *c_void, stream: *c_void, buf_in: *uv_buf_t, buf_cnt: c_int,
                     cb: uv_write_cb) -> c_int;
    fn rust_uv_read_start(stream: *c_void,
                          on_alloc: uv_alloc_cb,
                          on_read: uv_read_cb) -> c_int;
    fn rust_uv_read_stop(stream: *c_void) -> c_int;
    fn rust_uv_timer_init(loop_handle: *c_void, timer_handle: *uv_timer_t) -> c_int;
    fn rust_uv_timer_start(timer_handle: *uv_timer_t, cb: uv_timer_cb, timeout: libc::uint64_t,
                           repeat: libc::uint64_t) -> c_int;
    fn rust_uv_timer_stop(handle: *uv_timer_t) -> c_int;
    fn rust_uv_fs_open(loop_ptr: *c_void, req: *uv_fs_t, path: *c_char,
                       flags: c_int, mode: c_int, cb: *u8) -> c_int;
    fn rust_uv_fs_unlink(loop_ptr: *c_void, req: *uv_fs_t, path: *c_char,
                       cb: *u8) -> c_int;
    fn rust_uv_fs_write(loop_ptr: *c_void, req: *uv_fs_t, fd: c_int,
                       buf: *c_void, len: c_uint, offset: i64, cb: *u8) -> c_int;
    fn rust_uv_fs_read(loop_ptr: *c_void, req: *uv_fs_t, fd: c_int,
                       buf: *c_void, len: c_uint, offset: i64, cb: *u8) -> c_int;
    fn rust_uv_fs_close(loop_ptr: *c_void, req: *uv_fs_t, fd: c_int,
                        cb: *u8) -> c_int;
    fn rust_uv_fs_stat(loop_ptr: *c_void, req: *uv_fs_t, path: *c_char, cb: *u8) -> c_int;
    fn rust_uv_fs_fstat(loop_ptr: *c_void, req: *uv_fs_t, fd: c_int, cb: *u8) -> c_int;
    fn rust_uv_fs_mkdir(loop_ptr: *c_void, req: *uv_fs_t, path: *c_char,
                        mode: c_int, cb: *u8) -> c_int;
    fn rust_uv_fs_rmdir(loop_ptr: *c_void, req: *uv_fs_t, path: *c_char,
                        cb: *u8) -> c_int;
    fn rust_uv_fs_readdir(loop_ptr: *c_void, req: *uv_fs_t, path: *c_char,
                        flags: c_int, cb: *u8) -> c_int;
    fn rust_uv_fs_req_cleanup(req: *uv_fs_t);
    fn rust_uv_populate_uv_stat(req_in: *uv_fs_t, stat_out: *uv_stat_t);
    fn rust_uv_get_result_from_fs_req(req: *uv_fs_t) -> c_int;
    fn rust_uv_get_ptr_from_fs_req(req: *uv_fs_t) -> *libc::c_void;
    fn rust_uv_get_loop_from_fs_req(req: *uv_fs_t) -> *uv_loop_t;
    fn rust_uv_get_loop_from_getaddrinfo_req(req: *uv_fs_t) -> *uv_loop_t;

    fn rust_uv_get_stream_handle_from_connect_req(connect_req: *uv_connect_t) -> *uv_stream_t;
    fn rust_uv_get_stream_handle_from_write_req(write_req: *uv_write_t) -> *uv_stream_t;
    fn rust_uv_get_loop_for_uv_handle(handle: *c_void) -> *c_void;
    fn rust_uv_get_data_for_uv_loop(loop_ptr: *c_void) -> *c_void;
    fn rust_uv_set_data_for_uv_loop(loop_ptr: *c_void, data: *c_void);
    fn rust_uv_get_data_for_uv_handle(handle: *c_void) -> *c_void;
    fn rust_uv_set_data_for_uv_handle(handle: *c_void, data: *c_void);
    fn rust_uv_get_data_for_req(req: *c_void) -> *c_void;
    fn rust_uv_set_data_for_req(req: *c_void, data: *c_void);
    fn rust_uv_get_base_from_buf(buf: uv_buf_t) -> *u8;
    fn rust_uv_get_len_from_buf(buf: uv_buf_t) -> size_t;
    fn rust_uv_getaddrinfo(loop_: *uv_loop_t, req: *uv_getaddrinfo_t,
                           getaddrinfo_cb: uv_getaddrinfo_cb,
                           node: *c_char, service: *c_char,
                           hints: *addrinfo) -> c_int;
    fn rust_uv_freeaddrinfo(ai: *addrinfo);
    fn rust_uv_spawn(loop_ptr: *c_void, outptr: *uv_process_t,
                     options: uv_process_options_t) -> c_int;
    fn rust_uv_process_kill(p: *uv_process_t, signum: c_int) -> c_int;
    fn rust_uv_process_pid(p: *uv_process_t) -> c_int;
    fn rust_set_stdio_container_flags(c: *uv_stdio_container_t, flags: c_int);
    fn rust_set_stdio_container_fd(c: *uv_stdio_container_t, fd: c_int);
    fn rust_set_stdio_container_stream(c: *uv_stdio_container_t,
                                       stream: *uv_stream_t);
    fn rust_uv_pipe_init(loop_ptr: *c_void, p: *uv_pipe_t, ipc: c_int) -> c_int;
}
