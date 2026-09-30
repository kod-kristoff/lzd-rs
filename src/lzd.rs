// /* Lzd - Educational decompressor for the lzip format
//    Copyright (C) 2013-2025 Antonio Diaz Diaz.
//
//    This program is free software. Redistribution and use in source and
//    binary forms, with or without modification, are permitted provided
//    that the following conditions are met:
//
//    1. Redistributions of source code must retain the above copyright
//    notice, this list of conditions, and the following disclaimer.
//
//    2. Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions, and the following disclaimer in the
//    documentation and/or other materials provided with the distribution.
//
//    This program is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// */
// /*
//    Exit status: 0 for a normal exit, 1 for environmental problems
//    (file not found, invalid command-line options, I/O errors, etc), 2 to
//    indicate a corrupt or invalid input file.
// */
use std::io::{self, Read};
//
// #include <algorithm>
// #include <cerrno>
// #include <cstdio>
// #include <cstdlib>
// #include <cstring>
// #include <stdint.h>
// #include <unistd.h>
// #if defined __MSVCRT__ || defined __OS2__ || defined __DJGPP__
// #include <fcntl.h>
// #include <io.h>
// #endif
//
//
#[derive(Debug, Default, Clone, Copy)]
struct State {
    st: usize,
}

impl State {
    const states: usize = 12;
    fn get(&self) -> usize {
        return self.st;
    }
    fn is_char(&self) -> bool {
        return self.st < 7;
    }
}
// public:
//   enum { states = 12 };
//   State() : st( 0 ) {}
//   int operator()() const { return st; }
//   bool is_char() const { return st < 7; }
//
//   void set_char()
//     {
//     const int next[states] = { 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 4, 5 };
//     st = next[st];
//     }
//   void set_match()     { st = ( st < 7 ) ? 7 : 10; }
//   void set_rep()       { st = ( st < 7 ) ? 8 : 11; }
//   void set_short_rep() { st = ( st < 7 ) ? 9 : 11; }
//   };
//
//
// enum {
const min_dictionary_size: usize = 1 << 12;
const max_dictionary_size: usize = 1 << 29;
const literal_context_bits: usize = 3;
//   literal_pos_state_bits = 0,				// not used
const pos_state_bits: usize = 2;
const pos_states: usize = 1 << pos_state_bits;
const pos_state_mask: usize = pos_states - 1;
//
//   len_states = 4,
//   dis_slot_bits = 6,
//   start_dis_model = 4,
//   end_dis_model = 14,
//   modeled_distances = 1 << ( end_dis_model / 2 ),	// 128
//   dis_align_bits = 4,
//   dis_align_size = 1 << dis_align_bits,
//
//   len_low_bits = 3,
//   len_mid_bits = 3,
//   len_high_bits = 8,
//   len_low_symbols = 1 << len_low_bits,
//   len_mid_symbols = 1 << len_mid_bits,
//   len_high_symbols = 1 << len_high_bits,
//   max_len_symbols = len_low_symbols + len_mid_symbols + len_high_symbols,
//
//   min_match_len = 2,					// must be 2
//
const bit_model_move_bits: u32 = 5;
const bit_model_total_bits: u32 = 11;
const bit_model_total: u32 = 1 << bit_model_total_bits;
//};
//
#[derive(Debug, Clone, Copy)]
pub struct Bit_model {
    pub probability: u32,
}

impl Default for Bit_model {
    fn default() -> Self {
        Self {
            probability: bit_model_total / 2,
        }
    }
}
// struct Bit_model
//   {
//   int probability;
//   Bit_model() : probability( bit_model_total / 2 ) {}
//   };
//
// struct Len_model
//   {
//   Bit_model choice1;
//   Bit_model choice2;
//   Bit_model bm_low[pos_states][len_low_symbols];
//   Bit_model bm_mid[pos_states][len_mid_symbols];
//   Bit_model bm_high[len_high_symbols];
//   };
//
//
// class CRC32
struct CRC32 {
    data: [u32; 256], // Table of CRCs of all 8-bit messages.
}
impl CRC32 {
    pub const fn new() -> Self {
        let mut data = [0u32; 256];
        let mut n = 0u32;
        while n < 256 {
            let mut c = n;
            let mut k = 0;
            while k < 8 {
                if c & 1 != 0 {
                    c = 0xEDB88320 ^ (c >> 1);
                } else {
                    c >>= 1;
                }
                k += 1;
            }
            data[n as usize] = c;
            n += 1;
        }
        Self { data }
    }

    fn update_buf(&self, crc: &mut u32, buffer: &[u8]) {
        for i in 0..buffer.len() {
            *crc = self.data[((*crc ^ buffer[i] as u32) & 0xFF) as usize] ^ (*crc >> 8);
        }
    }
    //
    //   void update_buf( uint32_t & crc, const uint8_t * const buffer,
    //                    const int size ) const
    //     {
    //     for( int i = 0; i < size; ++i )
    //       crc = data[(crc^buffer[i])&0xFF] ^ ( crc >> 8 );
    //     }
    //   };
}
// const CRC32 crc32;
const crc32: CRC32 = CRC32::new();

const header_size: usize = 6;
type Lzip_header = [u8; header_size]; // 0-3 magic bytes
// 4 version
// 5 coded dictionary size
// enum { header_size = 6, trailer_size = 20 };
// typedef uint8_t Lzip_header[header_size]; // 0-3 magic bytes
// 					  //   4 version
// 					  //   5 coded dictionary size
// typedef uint8_t Lzip_trailer[trailer_size];
// 			//  0-3  CRC32 of the uncompressed data
// 			//  4-11 size of the uncompressed data
// 			// 12-19 member size including header and trailer
//
struct Range_decoder {
    member_pos: u64,
    code: u32,
    range: u32,
}

impl Range_decoder {
    pub fn new() -> Self
//   Range_decoder()
    //     : member_pos( header_size ), code( 0 ), range( 0xFFFFFFFFU )
    {
        let mut range_decoder = Self {
            member_pos: header_size as u64,
            code: 0,
            range: 0xFFFFFFFF,
        };
        // check first LZMA byte
        if range_decoder.get_byte() != 0 {
            eprint!("Nonzero first LZMA byte.\n");
            std::process::exit(2);
        }
        //     for( int i = 0; i < 4; ++i ) code = ( code << 8 ) | get_byte();
        for i in 0..4 {
            range_decoder.code = (range_decoder.code << 8) | range_decoder.get_byte() as u32;
        }
        range_decoder
    }
    //
    //   uint8_t get_byte() { ++member_pos; return std::getc( stdin ); }
    fn get_byte(&mut self) -> u8 {
        self.member_pos += 1;
        let mut c = [0; 1];
        io::stdin().read_exact(&mut c).unwrap();
        c[0]
    }
    //   unsigned long long member_position() const { return member_pos; }
    //
    //   unsigned decode( const int num_bits )
    //     {
    //     unsigned symbol = 0;
    //     for( int i = num_bits; i > 0; --i )
    //       {
    //       range >>= 1;
    //       symbol <<= 1;
    //       if( code >= range ) { code -= range; symbol |= 1; }
    //       if( range <= 0x00FFFFFFU )			// normalize
    //         { range <<= 8; code = ( code << 8 ) | get_byte(); }
    //       }
    //     return symbol;
    //     }
    //
    //   bool decode_bit( Bit_model & bm )
    pub fn decode_bit(&mut self, bm: &mut Bit_model) -> bool {
        //     bool symbol;
        //     const uint32_t bound = ( range >> bit_model_total_bits ) * bm.probability;
        let bound: u32 = (self.range >> bit_model_total_bits) * bm.probability;
        let symbol = if self.code < bound {
            self.range = bound;
            bm.probability += (bit_model_total - bm.probability) >> bit_model_move_bits;
            false
        } else {
            true
        };
        //     if( code < bound )
        //       {
        //       range = bound;
        //       bm.probability +=
        //         ( bit_model_total - bm.probability ) >> bit_model_move_bits;
        //       symbol = 0;
        //       }
        //     else
        //       {
        //       code -= bound;
        //       range -= bound;
        //       bm.probability -= bm.probability >> bit_model_move_bits;
        //       symbol = 1;
        //       }
        //     if( range <= 0x00FFFFFFU )				// normalize
        //       { range <<= 8; code = ( code << 8 ) | get_byte(); }
        return symbol;
    }
    //
    //   unsigned decode_tree( Bit_model bm[], const int num_bits )
    //     {
    //     unsigned symbol = 1;
    //     for( int i = 0; i < num_bits; ++i )
    //       symbol = ( symbol << 1 ) | decode_bit( bm[symbol] );
    //     return symbol - ( 1 << num_bits );
    //     }
    //
    //   unsigned decode_tree_reversed( Bit_model bm[], const int num_bits )
    //     {
    //     unsigned symbol = decode_tree( bm, num_bits );
    //     unsigned reversed_symbol = 0;
    //     for( int i = 0; i < num_bits; ++i )
    //       {
    //       reversed_symbol = ( reversed_symbol << 1 ) | ( symbol & 1 );
    //       symbol >>= 1;
    //       }
    //     return reversed_symbol;
    //     }
    //
    //   unsigned decode_matched( Bit_model bm[], const unsigned match_byte )
    //     {
    //     unsigned symbol = 1;
    //     for( int i = 7; i >= 0; --i )
    //       {
    //       const bool match_bit = ( match_byte >> i ) & 1;
    //       const bool bit = decode_bit( bm[symbol+(match_bit<<8)+0x100] );
    //       symbol = ( symbol << 1 ) | bit;
    //       if( match_bit != bit )
    //         {
    //         while( symbol < 0x100 )
    //           symbol = ( symbol << 1 ) | decode_bit( bm[symbol] );
    //         break;
    //         }
    //       }
    //     return symbol & 0xFF;
    //     }
    //
    //   unsigned decode_len( Len_model & lm, const int pos_state )
    //     {
    //     if( decode_bit( lm.choice1 ) == 0 )
    //       return min_match_len +
    //              decode_tree( lm.bm_low[pos_state], len_low_bits );
    //     if( decode_bit( lm.choice2 ) == 0 )
    //       return min_match_len + len_low_symbols +
    //              decode_tree( lm.bm_mid[pos_state], len_mid_bits );
    //     return min_match_len + len_low_symbols + len_mid_symbols +
    //            decode_tree( lm.bm_high, len_high_bits );
    //     }
    //   };
}
//
struct LZ_decoder {
    partial_data_pos: usize,
    rdec: Range_decoder,
    dictionary_size: usize,
    buffer: Box<[u8]>, // output buffer
    pos: usize,        // current pos in buffer
    stream_pos: usize, // first byte not yet written to stdout
    crc_: u32,
    pos_wrapped: bool,
}
//   void flush_data();
impl LZ_decoder {
    //   uint8_t peek( const unsigned distance ) const
    fn peek(&self, distance: usize) -> u8 {
        if self.pos > distance {
            return self.buffer[self.pos - distance - 1];
        }
        if self.pos_wrapped {
            return self.buffer[self.dictionary_size + self.pos - distance - 1];
        }
        return 0;
    }
    //     {
    //     if( pos > distance ) return buffer[pos - distance - 1];
    //     if( pos_wrapped ) return buffer[dictionary_size + pos - distance - 1];
    //     return 0;			// prev_byte of first byte
    //     }
    //
    //   void put_byte( const uint8_t b )
    fn put_byte(&mut self, b: u8) {
        self.buffer[self.pos] = b;
        self.pos += 1;
        if self.pos >= self.dictionary_size {
            self.flush_data();
        }
    }
    //
    // public:
    //   explicit LZ_decoder( const unsigned dict_size )
    pub fn new(dict_size: usize) -> Self {
        let dictionary_size = dict_size;
        Self {
            partial_data_pos: 0,
            dictionary_size: dict_size,
            buffer: vec![0; dictionary_size].into_boxed_slice(),
            pos: 0,
            stream_pos: 0,
            crc_: 0xFFFFFFFFu32,
            pos_wrapped: false,
            rdec: Range_decoder::new(),
        }
    }
    //
    //   ~LZ_decoder() { delete[] buffer; }
    //
    //   unsigned crc() const { return crc_ ^ 0xFFFFFFFFU; }
    fn data_position(&self) -> usize {
        return self.partial_data_pos + self.pos;
    }
    //   uint8_t get_byte() { return rdec.get_byte(); }
    //   unsigned long long member_position() const
    //     { return rdec.member_position(); }
    //
    //   bool decode_member();
    //   };
    //
    //
    // void LZ_decoder::flush_data()
    fn flush_data(&mut self) {
        if (self.pos > self.stream_pos) {
            let size = self.pos - self.stream_pos;
            crc32.update_buf(&mut self.crc_, &self.buffer[self.stream_pos..size]);
            //     const unsigned size = pos - stream_pos;
            //     crc32.update_buf( crc_, buffer + stream_pos, size );
            //     if( std::fwrite( buffer + stream_pos, 1, size, stdout ) != size )
            //       { std::fprintf( stderr, "Write error: %s\n", std::strerror( errno ) );
            //         std::exit( 1 ); }
            //     if( pos >= dictionary_size )
            //       { partial_data_pos += pos; pos = 0; pos_wrapped = true; }
            //     stream_pos = pos;
        }
    }
    //
    //
    pub fn decode_member(&mut self) -> bool // Return false if error
    {
        let bm_literal: [[Bit_model; 0x300]; 1 << literal_context_bits] =
            [[Bit_model::default(); 0x300]; 1 << literal_context_bits];
        let mut bm_match: [[Bit_model; pos_states]; State::states] = Default::default();
        //   Bit_model bm_literal[1<<literal_context_bits][0x300];
        //   Bit_model bm_match[State::states][pos_states];
        //   Bit_model bm_rep[State::states];
        //   Bit_model bm_rep0[State::states];
        //   Bit_model bm_rep1[State::states];
        //   Bit_model bm_rep2[State::states];
        //   Bit_model bm_len[State::states][pos_states];
        //   Bit_model bm_dis_slot[len_states][1<<dis_slot_bits];
        //   Bit_model bm_dis[modeled_distances-end_dis_model+1];
        //   Bit_model bm_align[dis_align_size];
        //   Len_model match_len_model;
        //   Len_model rep_len_model;
        let rep0 = 0; // rep[0-3] latest four distances
        let rep1 = 0; // used for efficient coding of
        let rep2 = 0; // repeated distances
        let rep3 = 0;
        let state = State::default();
        //
        //   while( !std::feof( stdin ) && !std::ferror( stdin ) )
        loop {
            let pos_state = self.data_position() & pos_state_mask;
            // 1st bit
            if !self.rdec.decode_bit(&mut bm_match[state.get()][pos_state]) {
                // literal byte
                let prev_byte = self.peek(0);
                let literal_state = prev_byte as usize >> (8 - literal_context_bits);
                let bm: &[Bit_model] = &bm_literal[literal_state][..];
                if state.is_char() {
                    self.put_byte(rdec.decode_tree(bm, 8));
                } else {
                    self.put_byte(rdec.decode_matched(bm, self.peek(rep0)));
                }
            }
            //     if( rdec.decode_bit( bm_match[state()][pos_state] ) == 0 )	// 1st bit
            //       {
            //       // literal byte
            //       const uint8_t prev_byte = peek( 0 );
            //       const int literal_state = prev_byte >> ( 8 - literal_context_bits );
            //       Bit_model * const bm = bm_literal[literal_state];
            //       if( state.is_char() )
            //         put_byte( rdec.decode_tree( bm, 8 ) );
            //       else
            //         put_byte( rdec.decode_matched( bm, peek( rep0 ) ) );
            //       state.set_char();
            //       continue;
            //       }
            //     // match or repeated match
            //     int len;
            //     if( rdec.decode_bit( bm_rep[state()] ) != 0 )		// 2nd bit
            //       {
            //       if( rdec.decode_bit( bm_rep0[state()] ) == 0 )		// 3rd bit
            //         {
            //         if( rdec.decode_bit( bm_len[state()][pos_state] ) == 0 ) // 4th bit
            //           { state.set_short_rep(); put_byte( peek( rep0 ) ); continue; }
            //         }
            //       else
            //         {
            //         unsigned distance;
            //         if( rdec.decode_bit( bm_rep1[state()] ) == 0 )		// 4th bit
            //           distance = rep1;
            //         else
            //           {
            //           if( rdec.decode_bit( bm_rep2[state()] ) == 0 )	// 5th bit
            //             distance = rep2;
            //           else
            //             { distance = rep3; rep3 = rep2; }
            //           rep2 = rep1;
            //           }
            //         rep1 = rep0;
            //         rep0 = distance;
            //         }
            //       state.set_rep();
            //       len = rdec.decode_len( rep_len_model, pos_state );
            //       }
            //     else					// match
            //       {
            //       rep3 = rep2; rep2 = rep1; rep1 = rep0;
            //       len = rdec.decode_len( match_len_model, pos_state );
            //       const int len_state = std::min( len - min_match_len, len_states - 1 );
            //       rep0 = rdec.decode_tree( bm_dis_slot[len_state], dis_slot_bits );
            //       if( rep0 >= start_dis_model )
            //         {
            //         const unsigned dis_slot = rep0;
            //         const int direct_bits = ( dis_slot >> 1 ) - 1;
            //         rep0 = ( 2 | ( dis_slot & 1 ) ) << direct_bits;
            //         if( dis_slot < end_dis_model )
            //           rep0 += rdec.decode_tree_reversed( bm_dis + ( rep0 - dis_slot ),
            //                                              direct_bits );
            //         else
            //           {
            //           rep0 += rdec.decode( direct_bits-dis_align_bits ) << dis_align_bits;
            //           rep0 += rdec.decode_tree_reversed( bm_align, dis_align_bits );
            //           if( rep0 == 0xFFFFFFFFU )		// marker found
            //             {
            //             flush_data();
            //             return len == min_match_len;	// End Of Stream marker
            //             }
            //           }
            //         }
            //       state.set_match();
            //       if( rep0 >= dictionary_size || ( rep0 >= pos && !pos_wrapped ) )
            //         { flush_data(); return false; }
            //       }
            //     for( int i = 0; i < len; ++i ) put_byte( peek( rep0 ) );
        }
        //   flush_data();
        return false;
    }
}
//
const PROGVERSION: &str = "0.1";
// int main( const int argc, const char * const argv[] )
fn main_(argc: usize, argv: Vec<String>) -> i32 {
    //   if( argc > 2 || ( argc == 2 && std::strcmp( argv[1], "-d" ) != 0 ) )
    if argc > 2 || (argc == 2 && argv[1] != "-d") {
        print!(
            "
Lzd {} - Educational decompressor for the lzip format.\n
Study the source code to learn how a lzip decompressor works.\n
See the lzip manual for an explanation of the code.\n
\nUsage: {} [-d] < file.lz > file\n
Lzd decompresses from standard input to standard output.\n
\nCopyright (C) 2025 Antonio Diaz Diaz.\n
License 2-clause BSD.\n
This is free software: you are free to change and redistribute it.\n
There is NO WARRANTY, to the extent permitted by law.\n
Report bugs to lzip-bug@nongnu.org\n
Lzd home page: http://www.nongnu.org/lzip/lzd.html\n",
            PROGVERSION, argv[0]
        );
        return 0;
    }
    //
    // #if defined __MSVCRT__ || defined __OS2__ || defined __DJGPP__
    //   setmode( STDIN_FILENO, O_BINARY );
    //   setmode( STDOUT_FILENO, O_BINARY );
    // #endif
    //
    let mut empty = false;
    let mut multi = false;
    let mut first_member = true;
    //   bool empty = false, multi = false;
    //   for( bool first_member = true; ; first_member = false )
    loop {
        let mut header: Lzip_header = Default::default(); // check header
        let res = io::stdin().read_exact(&mut header);

        // dbg!(&res);
        // dbg!(&header[0..5]);
        // dbg!(String::from_utf8_lossy(&header));
        //     for( int i = 0; i < header_size; ++i ) header[i] = std::getc( stdin );
        //     if( std::feof( stdin ) || std::memcmp( header, "LZIP\x01", 5 ) != 0 )
        if res.is_err() || &header[0..5] != b"LZIP\x01" {
            if (first_member) {
                eprint!("Bad magic number (file not in lzip format).\n");
                return 2;
            }
            break; // ignore trailing data
        }
        // todo!()
        //     unsigned dict_size = 1 << ( header[5] & 0x1F );
        let mut dict_size = 1 << (header[5] as usize & 0x1F);
        dict_size -= (dict_size / 16) * ((header[5] as usize >> 5) & 7);
        if (dict_size < min_dictionary_size || dict_size > max_dictionary_size) {
            eprint!("Invalid dictionary size in member header.\n");
            return 2;
        }
        //
        // decode LZMA stream
        let mut decoder = LZ_decoder::new(dict_size);
        if (!decoder.decode_member()) {
            eprint!("Data error.\n");
            return 2;
        }
        //
        //     Lzip_trailer trailer;			// check trailer
        //     for( int i = 0; i < trailer_size; ++i ) trailer[i] = decoder.get_byte();
        //     int retval = 0;
        //     unsigned crc = 0;
        //     for( int i = 3; i >= 0; --i ) crc = ( crc << 8 ) + trailer[i];
        //     if( crc != decoder.crc() )
        //       { std::fputs( "CRC mismatch.\n", stderr ); retval = 2; }
        //
        //     unsigned long long data_size = 0;
        //     for( int i = 11; i >= 4; --i )
        //       data_size = ( data_size << 8 ) + trailer[i];
        //     if( data_size != decoder.data_position() )
        //       { std::fputs( "Data size mismatch.\n", stderr ); retval = 2; }
        //     multi = !first_member; if( data_size == 0 ) empty = true;
        //
        //     unsigned long long member_size = 0;
        //     for( int i = 19; i >= 12; --i )
        //       member_size = ( member_size << 8 ) + trailer[i];
        //     if( member_size != decoder.member_position() )
        //       { std::fputs( "Member size mismatch.\n", stderr ); retval = 2; }
        //     if( retval ) return retval;
        first_member = false;
    }
    //
    //   if( std::fclose( stdout ) != 0 )
    //     { std::fprintf( stderr, "Error closing stdout: %s\n",
    //                     std::strerror( errno ) ); return 1; }
    //   if( empty && multi )
    //     { std::fputs( "Empty member not allowed.\n", stderr ); return 2; }
    return 0;
}

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let argc = argv.len();
    let exit_code = main_(argc, argv);
    std::process::exit(exit_code)
}
