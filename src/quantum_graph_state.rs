use std::collections::{BTreeSet, HashMap};
use indexmap::set::IndexSet;
use rand::Rng;
use rand::rngs::ThreadRng;

use crate::quantum_state::{Entropy, QuantumState};


const CONJUGATION_TABLE: [usize; 24] = [3, 6, 6, 3, 1, 1, 4, 4, 5, 2, 5, 2, 1, 1, 4, 4, 5, 2, 5, 2, 3, 6, 6, 3];

const IDGATE: usize     =  0;
const XGATE: usize      =  1;
const YGATE: usize      =  2;
const ZGATE: usize      =  3;
const HGATE: usize      = 12;
const SGATE: usize      = 20;
const SDGATE: usize     = 23;
const SQRTXGATE: usize  = 17;
const SQRTXDGATE: usize = 16;
const SQRTYGATE: usize  = 15;
const SQRTYDGATE: usize = 13;
const SQRTZGATE: usize  = 20;
const SQRTZDGATE: usize = 23;

const ZGATES: [usize; 4] = [IDGATE, ZGATE, SGATE, SDGATE];

const CLIFFORD_DECOMPS: [[usize; 5]; 24] =
   [[IDGATE, IDGATE, IDGATE, IDGATE, IDGATE],
	[SQRTXGATE, SQRTXGATE, IDGATE, IDGATE, IDGATE],
	[SQRTZGATE, SQRTZGATE, SQRTXGATE, SQRTXGATE, IDGATE],
	[SQRTZGATE, SQRTZGATE, IDGATE, IDGATE, IDGATE],
	[SQRTZGATE, SQRTZGATE, SQRTZGATE, SQRTXGATE, IDGATE],
	[SQRTZGATE, SQRTXGATE, SQRTXGATE, SQRTXGATE, IDGATE],
	[SQRTZGATE, SQRTXGATE, SQRTZGATE, SQRTZGATE, IDGATE],
	[SQRTZGATE, SQRTXGATE, IDGATE, IDGATE, IDGATE],
	[SQRTXGATE, SQRTXGATE, SQRTXGATE, SQRTZGATE, IDGATE],
	[SQRTXGATE, SQRTZGATE, IDGATE, IDGATE, IDGATE],
	[SQRTZGATE, SQRTXGATE, SQRTZGATE, SQRTXGATE, IDGATE],
	[SQRTXGATE, SQRTZGATE, SQRTZGATE, SQRTZGATE, IDGATE],
	[SQRTXGATE, SQRTZGATE, SQRTZGATE, SQRTZGATE, SQRTXGATE],
	[SQRTXGATE, SQRTZGATE, SQRTXGATE, SQRTXGATE, SQRTXGATE],
	[SQRTZGATE, SQRTXGATE, SQRTZGATE, IDGATE, IDGATE],
	[SQRTZGATE, SQRTXGATE, SQRTZGATE, SQRTZGATE, SQRTZGATE],
	[SQRTXGATE, SQRTXGATE, SQRTXGATE, IDGATE, IDGATE],
	[SQRTXGATE, IDGATE, IDGATE, IDGATE, IDGATE],
	[SQRTZGATE, SQRTZGATE, SQRTXGATE, IDGATE, IDGATE],
	[SQRTZGATE, SQRTZGATE, SQRTXGATE, SQRTXGATE, SQRTXGATE],
	[SQRTZGATE, SQRTZGATE, SQRTZGATE, IDGATE, IDGATE],
	[SQRTXGATE, SQRTXGATE, SQRTZGATE, SQRTZGATE, SQRTZGATE],
	[SQRTXGATE, SQRTXGATE, SQRTZGATE, IDGATE, IDGATE],
	[SQRTZGATE, IDGATE, IDGATE, IDGATE, IDGATE]];

const CLIFFORD_PRODUCTS: [[usize; 24]; 24] = 
   [[ 0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23],
	[ 1,  0,  3,  2,  6,  7,  4,  5, 11, 10,  9,  8, 15, 14, 13, 12, 17, 16, 19, 18, 22, 23, 20, 21],
	[ 2,  3,  0,  1,  7,  6,  5,  4,  9,  8, 11, 10, 14, 15, 12, 13, 19, 18, 17, 16, 21, 20, 23, 22],
	[ 3,  2,  1,  0,  5,  4,  7,  6, 10, 11,  8,  9, 13, 12, 15, 14, 18, 19, 16, 17, 23, 22, 21, 20],
	[ 4,  5,  6,  7,  8,  9, 10, 11,  0,  1,  2,  3, 23, 22, 21, 20, 13, 12, 15, 14, 18, 19, 16, 17],
	[ 5,  4,  7,  6, 10, 11,  8,  9,  3,  2,  1,  0, 20, 21, 22, 23, 12, 13, 14, 15, 16, 17, 18, 19],
	[ 6,  7,  4,  5, 11, 10,  9,  8,  1,  0,  3,  2, 21, 20, 23, 22, 14, 15, 12, 13, 19, 18, 17, 16],
	[ 7,  6,  5,  4,  9,  8, 11, 10,  2,  3,  0,  1, 22, 23, 20, 21, 15, 14, 13, 12, 17, 16, 19, 18],
	[ 8,  9, 10, 11,  0,  1,  2,  3,  4,  5,  6,  7, 17, 16, 19, 18, 22, 23, 20, 21, 15, 14, 13, 12],
	[ 9,  8, 11, 10,  2,  3,  0,  1,  7,  6,  5,  4, 18, 19, 16, 17, 23, 22, 21, 20, 13, 12, 15, 14],
	[10, 11,  8,  9,  3,  2,  1,  0,  5,  4,  7,  6, 19, 18, 17, 16, 21, 20, 23, 22, 14, 15, 12, 13],
	[11, 10,  9,  8,  1,  0,  3,  2,  6,  7,  4,  5, 16, 17, 18, 19, 20, 21, 22, 23, 12, 13, 14, 15],
	[12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,  0,  1,  2,  3,  4,  5, 6,  7,  8,  9,  10, 11],
	[13, 12, 15, 14, 18, 19, 16, 17, 23, 22, 21, 20,  3,  2,  1,  0,  5,  4, 7,  6,  10, 11,  8,  9],
	[14, 15, 12, 13, 19, 18, 17, 16, 21, 20, 23, 22,  2,  3,  0,  1,  7,  6, 5,  4,   9,  8, 11, 10],
	[15, 14, 13, 12, 17, 16, 19, 18, 22, 23, 20, 21,  1,  0,  3,  2,  6,  7, 4,  5,  11, 10,  9,  8],
	[16, 17, 18, 19, 20, 21, 22, 23, 12, 13, 14, 15, 11, 10,  9,  8,  1,  0, 3,  2,   6,  7,  4,  5],
	[17, 16, 19, 18, 22, 23, 20, 21, 15, 14, 13, 12,  8,  9, 10, 11,  0,  1, 2,  3,   4,  5,  6,  7],
	[18, 19, 16, 17, 23, 22, 21, 20, 13, 12, 15, 14,  9,  8, 11, 10,  2,  3, 0,  1,   7,  6,  5,  4],
	[19, 18, 17, 16, 21, 20, 23, 22, 14, 15, 12, 13, 10, 11,  8,  9,  3,  2, 1,  0,   5,  4,  7,  6],
	[20, 21, 22, 23, 12, 13, 14, 15, 16, 17, 18, 19,  5,  4,  7,  6, 10, 11, 8,  9,   3,  2,  1,  0],
	[21, 20, 23, 22, 14, 15, 12, 13, 19, 18, 17, 16,  6,  7,  4,  5, 11, 10, 9,  8,   1,  0,  3,  2],
	[22, 23, 20, 21, 15, 14, 13, 12, 17, 16, 19, 18,  7,  6,  5,  4,  9,  8, 11, 10,  2,  3,  0,  1],
	[23, 22, 21, 20, 13, 12, 15, 14, 18, 19, 16, 17,  4,  5,  6,  7,  8,  9, 10, 11,  0,  1,  2,  3]];

const CZ_LOOKUP: [[[(bool, usize, usize); 2]; 24]; 24] = 
  [[[(true, 0, 0), (false, 0, 0)], [(true, 0, 0), (false, 3, 0)], [(true, 0, 3), (false, 3, 2)], [(true, 0, 3), (false, 0, 3)],
	[(false, 0, 4), (true, 0, 5)], [(false, 0, 4), (true, 0, 4)], [(false, 3, 6), (true, 0, 6)], [(false, 3, 6), (true, 0, 7)],
	[(true, 0, 23), (false, 23, 8)], [(true, 0, 23), (false, 20, 8)], [(true, 0, 20), (false, 23, 10)], [(true, 0, 20), (false, 20, 10)],
	[(false, 0, 4), (true, 0, 13)], [(false, 0, 4), (true, 0, 12)], [(false, 3, 6), (true, 0, 14)], [(false, 3, 6), (true, 0, 15)],
	[(true, 0, 0), (false, 23, 0)], [(true, 0, 0), (false, 20, 0)], [(true, 0, 3), (false, 23, 2)], [(true, 0, 3), (false, 20, 2)],
	[(true, 0, 20), (false, 0, 20)], [(true, 0, 20), (false, 3, 10)], [(true, 0, 23), (false, 3, 8)], [(true, 0, 23), (false, 0, 23)]],
   [[(true, 0, 0), (false, 0, 3)], [(true, 0, 0), (false, 2, 2)], [(true, 0, 3), (false, 2, 0)], [(true, 0, 3), (false, 0, 0)],
	[(false, 0, 4), (true, 0, 7)], [(false, 0, 4), (true, 0, 6)], [(false, 2, 6), (true, 0, 4)], [(false, 2, 6), (true, 0, 5)],
	[(true, 0, 23), (false, 10, 10)], [(true, 0, 23), (false, 8, 10)], [(true, 0, 20), (false, 10, 8)], [(true, 0, 20), (false, 8, 8)],
	[(false, 0, 4), (true, 0, 15)], [(false, 0, 4), (true, 0, 14)], [(false, 2, 6), (true, 0, 12)], [(false, 2, 6), (true, 0, 13)],
	[(true, 0, 0), (false, 10, 2)], [(true, 0, 0), (false, 8, 2)], [(true, 0, 3), (false, 10, 0)], [(true, 0, 3), (false, 8, 0)],
	[(true, 0, 20), (false, 0, 23)], [(true, 0, 20), (false, 2, 8)], [(true, 0, 23), (false, 2, 10)], [(true, 0, 23), (false, 0, 20)]],
   [[(true, 2, 3), (false, 2, 3)], [(true, 0, 1), (false, 0, 2)], [(true, 0, 2), (false, 0, 0)], [(true, 2, 0), (false, 2, 0)],
	[(false, 2, 4), (true, 0, 6)], [(false, 2, 4), (true, 0, 7)], [(false, 0, 6), (true, 0, 5)], [(false, 0, 6), (true, 0, 4)],
	[(true, 0, 22), (false, 8, 10)], [(true, 0, 22), (false, 10, 10)], [(true, 0, 21), (false, 8, 8)], [(true, 0, 21), (false, 10, 8)],
	[(false, 2, 4), (true, 0, 14)], [(false, 2, 4), (true, 0, 15)], [(false, 0, 6), (true, 0, 13)], [(false, 0, 6), (true, 0, 12)],
	[(true, 0, 1), (false, 8, 2)], [(true, 0, 1), (false, 10, 2)], [(true, 0, 2), (false, 8, 0)], [(true, 0, 2), (false, 10, 0)],
	[(true, 2, 23), (false, 2, 23)], [(true, 0, 21), (false, 0, 8)], [(true, 0, 22), (false, 0, 10)], [(true, 2, 20), (false, 2, 20)]],
   [[(true, 3, 0), (false, 3, 0)], [(true, 0, 1), (false, 0, 0)], [(true, 0, 2), (false, 0, 2)], [(true, 3, 3), (false, 3, 3)],
	[(false, 3, 4), (true, 0, 4)], [(false, 3, 4), (true, 0, 5)], [(false, 0, 6), (true, 0, 7)], [(false, 0, 6), (true, 0, 6)],
	[(true, 0, 22), (false, 20, 8)], [(true, 0, 22), (false, 23, 8)], [(true, 0, 21), (false, 20, 10)], [(true, 0, 21), (false, 23, 10)],
	[(false, 3, 4), (true, 0, 12)], [(false, 3, 4), (true, 0, 13)], [(false, 0, 6), (true, 0, 15)], [(false, 0, 6), (true, 0, 14)],
	[(true, 0, 1), (false, 20, 0)], [(true, 0, 1), (false, 23, 0)], [(true, 0, 2), (false, 20, 2)], [(true, 0, 2), (false, 23, 2)],
	[(true, 3, 20), (false, 3, 20)], [(true, 0, 21), (false, 0, 10)], [(true, 0, 22), (false, 0, 8)], [(true, 3, 23), (false, 3, 23)]],
   [[(false, 4, 0), (true, 4, 3)], [(false, 4, 0), (true, 0, 6)], [(false, 4, 2), (true, 0, 7)], [(false, 4, 3), (true, 4, 0)],
	[(false, 4, 4), (false, 8, 8)], [(false, 4, 4), (false, 8, 10)], [(false, 4, 6), (false, 10, 10)], [(false, 4, 6), (false, 10, 8)],
	[(false, 4, 8), (false, 0, 0)], [(false, 4, 8), (false, 2, 2)], [(false, 4, 10), (false, 0, 2)], [(false, 4, 10), (false, 2, 0)],
	[(false, 4, 4), (false, 8, 0)], [(false, 4, 4), (false, 8, 2)], [(false, 4, 6), (false, 10, 2)], [(false, 4, 6), (false, 10, 0)],
	[(false, 4, 0), (false, 0, 10)], [(false, 4, 0), (false, 2, 8)], [(false, 4, 2), (false, 0, 8)], [(false, 4, 2), (false, 2, 10)],
	[(false, 4, 20), (true, 4, 23)], [(false, 4, 10), (true, 0, 14)], [(false, 4, 8), (true, 0, 15)], [(false, 4, 23), (true, 4, 20)]],
   [[(false, 4, 0), (true, 4, 0)], [(false, 4, 0), (true, 0, 7)], [(false, 4, 2), (true, 0, 6)], [(false, 4, 3), (true, 4, 3)],
	[(false, 4, 4), (false, 10, 8)], [(false, 4, 4), (false, 10, 10)], [(false, 4, 6), (false, 8, 10)], [(false, 4, 6), (false, 8, 8)],
	[(false, 4, 8), (false, 2, 0)], [(false, 4, 8), (false, 0, 2)], [(false, 4, 10), (false, 2, 2)], [(false, 4, 10), (false, 0, 0)],
	[(false, 4, 4), (false, 10, 0)], [(false, 4, 4), (false, 10, 2)], [(false, 4, 6), (false, 8, 2)], [(false, 4, 6), (false, 8, 0)],
	[(false, 4, 0), (false, 2, 10)], [(false, 4, 0), (false, 0, 8)], [(false, 4, 2), (false, 2, 8)], [(false, 4, 2), (false, 0, 10)],
	[(false, 4, 20), (true, 4, 20)], [(false, 4, 10), (true, 0, 15)], [(false, 4, 8), (true, 0, 14)], [(false, 4, 23), (true, 4, 23)]],
   [[(false, 6, 3), (true, 6, 0)], [(false, 6, 2), (true, 0, 4)], [(false, 6, 0), (true, 0, 5)], [(false, 6, 0), (true, 6, 3)],
	[(false, 6, 4), (false, 10, 10)], [(false, 6, 4), (false, 10, 8)], [(false, 6, 6), (false, 8, 8)], [(false, 6, 6), (false, 8, 10)],
	[(false, 6, 10), (false, 0, 2)], [(false, 6, 10), (false, 2, 0)], [(false, 6, 8), (false, 0, 0)], [(false, 6, 8), (false, 2, 2)],
	[(false, 6, 4), (false, 10, 2)], [(false, 6, 4), (false, 10, 0)], [(false, 6, 6), (false, 8, 0)], [(false, 6, 6), (false, 8, 2)],
	[(false, 6, 2), (false, 0, 8)], [(false, 6, 2), (false, 2, 10)], [(false, 6, 0), (false, 0, 10)], [(false, 6, 0), (false, 2, 8)],
	[(false, 6, 23), (true, 6, 20)], [(false, 6, 8), (true, 0, 12)], [(false, 6, 10), (true, 0, 13)], [(false, 6, 20), (true, 6, 23)]],
   [[(false, 6, 3), (true, 6, 3)], [(false, 6, 2), (true, 0, 5)], [(false, 6, 0), (true, 0, 4)], [(false, 6, 0), (true, 6, 0)],
	[(false, 6, 4), (false, 8, 10)], [(false, 6, 4), (false, 8, 8)], [(false, 6, 6), (false, 10, 8)], [(false, 6, 6), (false, 10, 10)],
	[(false, 6, 10), (false, 2, 2)], [(false, 6, 10), (false, 0, 0)], [(false, 6, 8), (false, 2, 0)], [(false, 6, 8), (false, 0, 2)],
	[(false, 6, 4), (false, 8, 2)], [(false, 6, 4), (false, 8, 0)], [(false, 6, 6), (false, 10, 0)], [(false, 6, 6), (false, 10, 2)],
	[(false, 6, 2), (false, 2, 8)], [(false, 6, 2), (false, 0, 10)], [(false, 6, 0), (false, 2, 10)], [(false, 6, 0), (false, 0, 8)],
	[(false, 6, 23), (true, 6, 23)], [(false, 6, 8), (true, 0, 13)], [(false, 6, 10), (true, 0, 12)], [(false, 6, 20), (true, 6, 20)]],
   [[(true, 8, 20), (false, 8, 23)], [(true, 0, 16), (false, 10, 10)], [(true, 0, 18), (false, 10, 8)], [(true, 8, 23), (false, 8, 20)],
	[(false, 8, 4), (false, 0, 0)], [(false, 8, 4), (false, 0, 2)], [(false, 10, 6), (false, 2, 0)], [(false, 10, 6), (false, 2, 2)],
	[(true, 0, 8), (true, 0, 5)], [(true, 0, 8), (true, 0, 7)], [(true, 0, 10), (true, 0, 4)], [(true, 0, 10), (true, 0, 6)],
	[(false, 8, 4), (false, 0, 10)], [(false, 8, 4), (false, 0, 8)], [(false, 10, 6), (false, 2, 10)], [(false, 10, 6), (false, 2, 8)],
	[(true, 0, 16), (true, 0, 13)], [(true, 0, 16), (true, 0, 15)], [(true, 0, 18), (true, 0, 12)], [(true, 0, 18), (true, 0, 14)],
	[(true, 8, 3), (false, 8, 0)], [(true, 0, 10), (false, 10, 2)], [(true, 0, 8), (false, 10, 0)], [(true, 8, 0), (false, 8, 3)]],
   [[(true, 8, 20), (false, 8, 20)], [(true, 0, 16), (false, 10, 8)], [(true, 0, 18), (false, 10, 10)], [(true, 8, 23), (false, 8, 23)],
	[(false, 8, 4), (false, 2, 2)], [(false, 8, 4), (false, 2, 0)], [(false, 10, 6), (false, 0, 2)], [(false, 10, 6), (false, 0, 0)],
	[(true, 0, 8), (true, 0, 6)], [(true, 0, 8), (true, 0, 4)], [(true, 0, 10), (true, 0, 7)], [(true, 0, 10), (true, 0, 5)],
	[(false, 8, 4), (false, 2, 8)], [(false, 8, 4), (false, 2, 10)], [(false, 10, 6), (false, 0, 8)], [(false, 10, 6), (false, 0, 10)],
	[(true, 0, 16), (true, 0, 14)], [(true, 0, 16), (true, 0, 12)], [(true, 0, 18), (true, 0, 15)], [(true, 0, 18), (true, 0, 13)],
	[(true, 8, 3), (false, 8, 3)], [(true, 0, 10), (false, 10, 0)], [(true, 0, 8), (false, 10, 2)], [(true, 8, 0), (false, 8, 0)]],
   [[(true, 10, 20), (false, 10, 23)], [(true, 0, 17), (false, 8, 10)], [(true, 0, 19), (false, 8, 8)], [(true, 10, 23), (false, 10, 20)],
	[(false, 10, 4), (false, 2, 0)], [(false, 10, 4), (false, 2, 2)], [(false, 8, 6), (false, 0, 0)], [(false, 8, 6), (false, 0, 2)],
	[(true, 0, 9), (true, 0, 4)], [(true, 0, 9), (true, 0, 6)], [(true, 0, 11), (true, 0, 5)], [(true, 0, 11), (true, 0, 7)],
	[(false, 10, 4), (false, 2, 10)], [(false, 10, 4), (false, 2, 8)], [(false, 8, 6), (false, 0, 10)], [(false, 8, 6), (false, 0, 8)],
	[(true, 0, 17), (true, 0, 12)], [(true, 0, 17), (true, 0, 14)], [(true, 0, 19), (true, 0, 13)], [(true, 0, 19), (true, 0, 15)],
	[(true, 10, 3), (false, 10, 0)], [(true, 0, 11), (false, 8, 2)], [(true, 0, 9), (false, 8, 0)], [(true, 10, 0), (false, 10, 3)]],
   [[(true, 10, 20), (false, 10, 20)], [(true, 0, 17), (false, 8, 8)], [(true, 0, 19), (false, 8, 10)], [(true, 10, 23), (false, 10, 23)],
	[(false, 10, 4), (false, 0, 2)], [(false, 10, 4), (false, 0, 0)], [(false, 8, 6), (false, 2, 2)], [(false, 8, 6), (false, 2, 0)],
	[(true, 0, 9), (true, 0, 7)], [(true, 0, 9), (true, 0, 5)], [(true, 0, 11), (true, 0, 6)], [(true, 0, 11), (true, 0, 4)],
	[(false, 10, 4), (false, 0, 8)], [(false, 10, 4), (false, 0, 10)], [(false, 8, 6), (false, 2, 8)], [(false, 8, 6), (false, 2, 10)],
	[(true, 0, 17), (true, 0, 15)], [(true, 0, 17), (true, 0, 13)], [(true, 0, 19), (true, 0, 14)], [(true, 0, 19), (true, 0, 12)],
	[(true, 10, 3), (false, 10, 3)], [(true, 0, 11), (false, 8, 0)], [(true, 0, 9), (false, 8, 2)], [(true, 10, 0), (false, 10, 0)]],
   [[(false, 4, 0), (true, 4, 23)], [(false, 4, 0), (true, 0, 15)], [(false, 4, 2), (true, 0, 14)], [(false, 4, 3), (true, 4, 20)],
	[(false, 4, 4), (false, 0, 8)], [(false, 4, 4), (false, 0, 10)], [(false, 4, 6), (false, 2, 10)], [(false, 4, 6), (false, 2, 8)],
	[(false, 4, 8), (false, 10, 0)], [(false, 4, 8), (false, 8, 2)], [(false, 4, 10), (false, 10, 2)], [(false, 4, 10), (false, 8, 0)],
	[(false, 4, 4), (false, 0, 0)], [(false, 4, 4), (false, 0, 2)], [(false, 4, 6), (false, 2, 2)], [(false, 4, 6), (false, 2, 0)],
	[(false, 4, 0), (false, 10, 10)], [(false, 4, 0), (false, 8, 8)], [(false, 4, 2), (false, 10, 8)], [(false, 4, 2), (false, 8, 10)],
	[(false, 4, 20), (true, 4, 0)], [(false, 4, 10), (true, 0, 6)], [(false, 4, 8), (true, 0, 7)], [(false, 4, 23), (true, 4, 3)]],
   [[(false, 4, 0), (true, 4, 20)], [(false, 4, 0), (true, 0, 14)], [(false, 4, 2), (true, 0, 15)], [(false, 4, 3), (true, 4, 23)],
	[(false, 4, 4), (false, 2, 8)], [(false, 4, 4), (false, 2, 10)], [(false, 4, 6), (false, 0, 10)], [(false, 4, 6), (false, 0, 8)],
	[(false, 4, 8), (false, 8, 0)], [(false, 4, 8), (false, 10, 2)], [(false, 4, 10), (false, 8, 2)], [(false, 4, 10), (false, 10, 0)],
	[(false, 4, 4), (false, 2, 0)], [(false, 4, 4), (false, 2, 2)], [(false, 4, 6), (false, 0, 2)], [(false, 4, 6), (false, 0, 0)],
	[(false, 4, 0), (false, 8, 10)], [(false, 4, 0), (false, 10, 8)], [(false, 4, 2), (false, 8, 8)], [(false, 4, 2), (false, 10, 10)], 
	[(false, 4, 20), (true, 4, 3)], [(false, 4, 10), (true, 0, 7)], [(false, 4, 8), (true, 0, 6)], [(false, 4, 23), (true, 4, 0)]],
   [[(false, 6, 3), (true, 6, 23)], [(false, 6, 2), (true, 0, 12)], [(false, 6, 0), (true, 0, 13)], [(false, 6, 0), (true, 6, 20)],
	[(false, 6, 4), (false, 2, 10)], [(false, 6, 4), (false, 2, 8)], [(false, 6, 6), (false, 0, 8)], [(false, 6, 6), (false, 0, 10)],
	[(false, 6, 10), (false, 10, 2)], [(false, 6, 10), (false, 8, 0)], [(false, 6, 8), (false, 10, 0)], [(false, 6, 8), (false, 8, 2)],
	[(false, 6, 4), (false, 2, 2)], [(false, 6, 4), (false, 2, 0)], [(false, 6, 6), (false, 0, 0)], [(false, 6, 6), (false, 0, 2)],
	[(false, 6, 2), (false, 10, 8)], [(false, 6, 2), (false, 8, 10)], [(false, 6, 0), (false, 10, 10)], [(false, 6, 0), (false, 8, 8)],
	[(false, 6, 23), (true, 6, 0)], [(false, 6, 8), (true, 0, 5)], [(false, 6, 10), (true, 0, 4)], [(false, 6, 20), (true, 6, 3)]],
   [[(false, 6, 3), (true, 6, 20)], [(false, 6, 2), (true, 0, 13)], [(false, 6, 0), (true, 0, 12)], [(false, 6, 0), (true, 6, 23)],
	[(false, 6, 4), (false, 0, 10)], [(false, 6, 4), (false, 0, 8)], [(false, 6, 6), (false, 2, 8)], [(false, 6, 6), (false, 2, 10)],
	[(false, 6, 10), (false, 8, 2)], [(false, 6, 10), (false, 10, 0)], [(false, 6, 8), (false, 8, 0)], [(false, 6, 8), (false, 10, 2)],
	[(false, 6, 4), (false, 0, 2)], [(false, 6, 4), (false, 0, 0)], [(false, 6, 6), (false, 2, 0)], [(false, 6, 6), (false, 2, 2)],
	[(false, 6, 2), (false, 8, 8)], [(false, 6, 2), (false, 10, 10)], [(false, 6, 0), (false, 8, 10)], [(false, 6, 0), (false, 10, 8)],
	[(false, 6, 23), (true, 6, 3)], [(false, 6, 8), (true, 0, 4)], [(false, 6, 10), (true, 0, 5)], [(false, 6, 20), (true, 6, 0)]],
   [[(true, 0, 0), (false, 0, 23)], [(true, 0, 0), (false, 2, 10)], [(true, 0, 3), (false, 2, 8)], [(true, 0, 3), (false, 0, 20)],
	[(false, 0, 4), (false, 10, 0)], [(false, 0, 4), (false, 10, 2)], [(false, 2, 6), (false, 8, 0)], [(false, 2, 6), (false, 8, 2)],
	[(true, 0, 23), (true, 0, 13)], [(true, 0, 23), (true, 0, 14)], [(true, 0, 20), (true, 0, 12)], [(true, 0, 20), (true, 0, 15)],
	[(false, 0, 4), (false, 10, 10)], [(false, 0, 4), (false, 10, 8)], [(false, 2, 6), (false, 8, 10)], [(false, 2, 6), (false, 8, 8)],
	[(true, 0, 0), (true, 0, 4)], [(true, 0, 0), (true, 0, 7)], [(true, 0, 3), (true, 0, 5)], [(true, 0, 3), (true, 0, 6)],
	[(true, 0, 20), (false, 0, 0)], [(true, 0, 20), (false, 2, 2)], [(true, 0, 23), (false, 2, 0)], [(true, 0, 23), (false, 0, 3)]],
   [[(true, 0, 0), (false, 0, 20)], [(true, 0, 0), (false, 2, 8)], [(true, 0, 3), (false, 2, 10)], [(true, 0, 3), (false, 0, 23)],
	[(false, 0, 4), (false, 8, 2)], [(false, 0, 4), (false, 8, 0)], [(false, 2, 6), (false, 10, 2)], [(false, 2, 6), (false, 10, 0)],
	[(true, 0, 23), (true, 0, 15)], [(true, 0, 23), (true, 0, 12)], [(true, 0, 20), (true, 0, 14)], [(true, 0, 20), (true, 0, 13)],
	[(false, 0, 4), (false, 8, 8)], [(false, 0, 4), (false, 8, 10)], [(false, 2, 6), (false, 10, 8)], [(false, 2, 6), (false, 10, 10)],
	[(true, 0, 0), (true, 0, 6)], [(true, 0, 0), (true, 0, 5)], [(true, 0, 3), (true, 0, 7)], [(true, 0, 3), (true, 0, 4)],
	[(true, 0, 20), (false, 0, 3)], [(true, 0, 20), (false, 2, 0)], [(true, 0, 23), (false, 2, 2)], [(true, 0, 23), (false, 0, 0)]],
   [[(true, 2, 3), (false, 2, 23)], [(true, 0, 1), (false, 0, 10)], [(true, 0, 2), (false, 0, 8)], [(true, 2, 0), (false, 2, 20)],
	[(false, 2, 4), (false, 8, 0)], [(false, 2, 4), (false, 8, 2)], [(false, 0, 6), (false, 10, 0)], [(false, 0, 6), (false, 10, 2)],
	[(true, 0, 22), (true, 0, 12)], [(true, 0, 22), (true, 0, 15)], [(true, 0, 21), (true, 0, 13)], [(true, 0, 21), (true, 0, 14)],
	[(false, 2, 4), (false, 8, 10)], [(false, 2, 4), (false, 8, 8)], [(false, 0, 6), (false, 10, 10)], [(false, 0, 6), (false, 10, 8)],
	[(true, 0, 1), (true, 0, 5)], [(true, 0, 1), (true, 0, 6)], [(true, 0, 2), (true, 0, 4)], [(true, 0, 2), (true, 0, 7)],
	[(true, 2, 23), (false, 2, 0)], [(true, 0, 21), (false, 0, 2)], [(true, 0, 22), (false, 0, 0)], [(true, 2, 20), (false, 2, 3)]],
   [[(true, 2, 3), (false, 2, 20)], [(true, 0, 1), (false, 0, 8)], [(true, 0, 2), (false, 0, 10)], [(true, 2, 0), (false, 2, 23)], 
	[(false, 2, 4), (false, 10, 2)], [(false, 2, 4), (false, 10, 0)], [(false, 0, 6), (false, 8, 2)], [(false, 0, 6), (false, 8, 0)],
	[(true, 0, 22), (true, 0, 14)], [(true, 0, 22), (true, 0, 13)], [(true, 0, 21), (true, 0, 15)], [(true, 0, 21), (true, 0, 12)],
	[(false, 2, 4), (false, 10, 8)], [(false, 2, 4), (false, 10, 10)], [(false, 0, 6), (false, 8, 8)], [(false, 0, 6), (false, 8, 10)],
	[(true, 0, 1), (true, 0, 7)], [(true, 0, 1), (true, 0, 4)], [(true, 0, 2), (true, 0, 6)], [(true, 0, 2), (true, 0, 5)],
	[(true, 2, 23), (false, 2, 3)], [(true, 0, 21), (false, 0, 0)], [(true, 0, 22), (false, 0, 2)], [(true, 2, 20), (false, 2, 0)]],
   [[(true, 20, 0), (false, 20, 0)], [(true, 0, 17), (false, 23, 0)], [(true, 0, 19), (false, 23, 2)], [(true, 20, 3), (false, 20, 3)],
	[(false, 20, 4), (true, 0, 13)], [(false, 20, 4), (true, 0, 12)], [(false, 23, 6), (true, 0, 15)], [(false, 23, 6), (true, 0, 14)],
	[(true, 0, 9), (false, 0, 8)], [(true, 0, 9), (false, 3, 8)], [(true, 0, 11), (false, 0, 10)], [(true, 0, 11), (false, 3, 10)],
	[(false, 20, 4), (true, 0, 4)], [(false, 20, 4), (true, 0, 5)], [(false, 23, 6), (true, 0, 6)], [(false, 23, 6), (true, 0, 7)],
	[(true, 0, 17), (false, 0, 0)], [(true, 0, 17), (false, 3, 0)], [(true, 0, 19), (false, 0, 2)], [(true, 0, 19), (false, 3, 2)],
	[(true, 20, 20), (false, 20, 20)], [(true, 0, 11), (false, 23, 10)], [(true, 0, 9), (false, 23, 8)], [(true, 20, 23), (false, 20, 23)]],
   [[(true, 10, 20), (false, 10, 3)], [(true, 0, 17), (false, 8, 2)], [(true, 0, 19), (false, 8, 0)], [(true, 10, 23), (false, 10, 0)],
	[(false, 10, 4), (true, 0, 14)], [(false, 10, 4), (true, 0, 15)], [(false, 8, 6), (true, 0, 12)], [(false, 8, 6), (true, 0, 13)],
	[(true, 0, 9), (false, 2, 10)], [(true, 0, 9), (false, 0, 10)], [(true, 0, 11), (false, 2, 8)], [(true, 0, 11), (false, 0, 8)],
	[(false, 10, 4), (true, 0, 7)], [(false, 10, 4), (true, 0, 6)], [(false, 8, 6), (true, 0, 5)], [(false, 8, 6), (true, 0, 4)],
	[(true, 0, 17), (false, 2, 2)], [(true, 0, 17), (false, 0, 2)], [(true, 0, 19), (false, 2, 0)], [(true, 0, 19), (false, 0, 0)],
	[(true, 10, 3), (false, 10, 23)], [(true, 0, 11), (false, 8, 8)], [(true, 0, 9), (false, 8, 10)], [(true, 10, 0), (false, 10, 20)]],
   [[(true, 8, 20), (false, 8, 3)], [(true, 0, 16), (false, 10, 2)], [(true, 0, 18), (false, 10, 0)], [(true, 8, 23), (false, 8, 0)],
	[(false, 8, 4), (true, 0, 15)], [(false, 8, 4), (true, 0, 14)], [(false, 10, 6), (true, 0, 13)], [(false, 10, 6), (true, 0, 12)],
	[(true, 0, 8), (false, 0, 10)], [(true, 0, 8), (false, 2, 10)], [(true, 0, 10), (false, 0, 8)], [(true, 0, 10), (false, 2, 8)],
	[(false, 8, 4), (true, 0, 6)], [(false, 8, 4), (true, 0, 7)], [(false, 10, 6), (true, 0, 4)], [(false, 10, 6), (true, 0, 5)],
	[(true, 0, 16), (false, 0, 2)], [(true, 0, 16), (false, 2, 2)], [(true, 0, 18), (false, 0, 0)], [(true, 0, 18), (false, 2, 0)],
	[(true, 8, 3), (false, 8, 23)], [(true, 0, 10), (false, 10, 8)], [(true, 0, 8), (false, 10, 10)], [(true, 8, 0), (false, 8, 20)]],
   [[(true, 23, 0), (false, 23, 0)], [(true, 0, 16), (false, 20, 0)], [(true, 0, 18), (false, 20, 2)], [(true, 23, 3), (false, 23, 3)],
	[(false, 23, 4), (true, 0, 12)], [(false, 23, 4), (true, 0, 13)], [(false, 20, 6), (true, 0, 14)], [(false, 20, 6), (true, 0, 15)],
	[(true, 0, 8), (false, 3, 8)], [(true, 0, 8), (false, 0, 8)], [(true, 0, 10), (false, 3, 10)], [(true, 0, 10), (false, 0, 10)],
	[(false, 23, 4), (true, 0, 5)], [(false, 23, 4), (true, 0, 4)], [(false, 20, 6), (true, 0, 7)], [(false, 20, 6), (true, 0, 6)],
	[(true, 0, 16), (false, 3, 0)], [(true, 0, 16), (false, 0, 0)], [(true, 0, 18), (false, 3, 2)], [(true, 0, 18), (false, 0, 2)],
	[(true, 23, 20), (false, 23, 20)], [(true, 0, 10), (false, 20, 10)], [(true, 0, 8), (false, 20, 8)], [(true, 23, 23), (false, 23, 23)]]];





pub struct Graph<T> {
	num_vertices: usize,
	vals: Vec<T>,
	edges: Vec<IndexSet<usize>>
}

impl<T: std::fmt::Display> Graph<T> {
	pub fn new() -> Graph<T> {
		Graph { num_vertices: 0, vals: Vec::new(), edges: Vec::new() }
	}

	pub fn with_capacity(num_vertices: usize) -> Graph<T> {
		Graph { num_vertices: num_vertices, vals: Vec::with_capacity(num_vertices), edges: Vec::with_capacity(num_vertices) }
	}

	pub fn print(&self) -> String {
		let mut s: String = String::new();
		for i in 0..self.num_vertices {
			s.push_str(&format!("[{}] {} -> ", self.vals[i], i));
			for j in &self.edges[i] {
				s += &String::from(j.to_string() + " ");
			}
			if i != self.num_vertices - 1 {
				s += "\n";
			}
		}
		return s;
	}

	pub fn add_vertex(&mut self, val: T) {
		self.num_vertices += 1;
		self.edges.push(IndexSet::new());
		self.vals.push(val);
	}

	pub fn remove_vertex(&mut self, vertex: usize) {
		self.num_vertices -= 1;
		self.vals.remove(vertex);
		self.edges.remove(vertex);
		for i in 0..self.num_vertices {
			self.edges[i].remove(&vertex);
		} 

		for i in 0..self.num_vertices {
			self.edges[i] = std::mem::take(&mut self.edges[i])
			.into_iter()
			.map(|x| if x > vertex { x - 1 } else { x})
			.collect();
		}
	}

	pub fn contains_edge(&self, vertex1: usize, vertex2: usize) -> bool {
		return self.edges[vertex1].contains(&vertex2) && self.edges[vertex2].contains(&vertex1);
	}

	pub fn add_edge(&mut self, vertex1: usize, vertex2: usize) {
		self.edges[vertex1].insert(vertex2);
		self.edges[vertex2].insert(vertex1);
	}

	pub fn remove_edge(&mut self, vertex1: usize, vertex2: usize) {
		self.edges[vertex1].remove(&vertex2);
		self.edges[vertex2].remove(&vertex1);
	}

	pub fn toggle_edge(&mut self, vertex1: usize, vertex2: usize) {
		if self.contains_edge(vertex1, vertex2) {
			self.remove_edge(vertex1, vertex2);
		} else {
			self.add_edge(vertex1, vertex2);
		}
	}

	pub fn degree(&self, vertex: usize) -> usize {
		return self.edges[vertex].len();
	}

	pub fn local_complement(&mut self, vertex: usize) {
		let mut toggles: Vec<(usize, usize)> = Vec::new();

		for i in &self.edges[vertex] {
			for j in &self.edges[vertex] {
				if i < j {
					toggles.push((*i, *j));
				}
			}
		}

		for t in &toggles {
			self.toggle_edge(t.0, t.1);
		}
	}

	pub fn partition(&self, set: &Vec<usize>) -> Graph<bool> {
		let mut new_graph: Graph<bool> = Graph::new();
		let mut new_vertices: HashMap<usize, usize> = HashMap::new(); // TODO IndexSet?
		for a in set {
			if self.degree(*a) == 0 { continue }

			new_vertices.insert(*a, new_vertices.len());
			new_graph.add_vertex(true);
			for b in &self.edges[*a] {
				if set.contains(&b) { continue }

				if !new_vertices.contains_key(b) {
					new_vertices.insert(*b, new_vertices.len());
					new_graph.add_vertex(false);
				}
				new_graph.add_edge(new_vertices[a], new_vertices[b]);
			}
		}

		// Delete isolated vertices
		let mut continue_deleting: bool = true;
		while continue_deleting {
			continue_deleting = false;
			for i in 0..new_graph.num_vertices {
				if new_graph.degree(i) == 0 {
					new_graph.remove_vertex(i);
					continue_deleting = true;
					break;
				}
			}
		}

		return new_graph;
	}
}

pub struct QuantumGraphState {
	num_qubits: usize,
	graph: Graph<usize>,
	rng: ThreadRng,
}

impl QuantumGraphState {
	fn apply_gate(&mut self, qubit: usize, gate_id: usize) {
		self.graph.vals[qubit] = CLIFFORD_PRODUCTS[gate_id][self.graph.vals[qubit]];
	}

	fn right_apply_gate(&mut self, qubit: usize, gate_id: usize) {
		self.graph.vals[qubit] = CLIFFORD_PRODUCTS[self.graph.vals[qubit]][gate_id];
	}

	fn local_complement(&mut self, qubit: usize) {
		self.graph.local_complement(qubit);

		// Apply correction
		self.right_apply_gate(qubit, SQRTXDGATE);
		for i in 0..self.graph.degree(qubit) {
			self.right_apply_gate(self.graph.edges[qubit][i], SQRTZGATE);
		}
	}

	fn remove_vop(&mut self, qubit1: usize, qubit2: usize) {
		let vop_decomp = CLIFFORD_DECOMPS[self.graph.vals[qubit1]];
		let mut c: usize = qubit2;

		for i in 0..self.graph.degree(qubit1) {
			if self.graph.edges[qubit1][i] != qubit2 {
				c = self.graph.edges[qubit1][i];
				break;
			}
		}

		for op in vop_decomp {
			match op {
				SQRTXGATE => self.local_complement(qubit1),
				SQRTZGATE => self.local_complement(c),
				_ => ()
			};
		}
	}

	fn isolated(&self, qubit1: usize, qubit2: usize) -> bool {
		let deg: usize = self.graph.degree(qubit1);
		if deg == 0 {
			return true;
		} else if deg == 1 {
			return self.graph.contains_edge(qubit1, qubit2);
		} else { 
			return false;
		}
	}

	fn mxr_graph(&mut self, qubit: usize, measured: i32) {
		let qubit_n = self.graph.edges[qubit][0];
		// Updating VOPs
		let neighbors_a: BTreeSet<usize> = self.graph.edges[qubit].iter().cloned().collect();
		let neighbors_b: BTreeSet<usize> = self.graph.edges[qubit_n].iter().cloned().collect();

		if measured == 1 {
			for n in &neighbors_b {
				if !(*n == qubit) && neighbors_a.contains(&n) {
					self.right_apply_gate(*n, ZGATE);
				}
			}
			self.right_apply_gate(qubit, ZGATE);
			self.right_apply_gate(qubit_n, SQRTYGATE);
		} else {
			for n in &neighbors_a {
				if !(*n == qubit_n) && neighbors_b.contains(&n) {
					self.right_apply_gate(*n, ZGATE);
				}
			}
			self.right_apply_gate(qubit_n, SQRTYDGATE);
		}

		// Updating graph edges
		for c in &neighbors_a {
			for d in &neighbors_b {
				self.graph.toggle_edge(*c, *d);
			}
		}

		for c in &neighbors_a {
			if neighbors_b.contains(&c) {
				for d in &neighbors_a {
					if neighbors_b.contains(&d) {
						self.graph.toggle_edge(*c, *d);
					}
				}
			}
		}

		for d in neighbors_a {
			if d != qubit_n {
				self.graph.toggle_edge(qubit_n, d);
			}
		}
		
	}

	fn myr_graph(&mut self, qubit: usize, measured: i32) {
		let gate_id = if measured == 1 { SGATE } else { SDGATE };

		// Updating VOPs
		let neighbors: Vec<usize> = self.graph.edges[qubit].iter().cloned().collect();
		for n in neighbors {
			self.right_apply_gate(n, gate_id);
		}
		self.right_apply_gate(qubit, gate_id);

		// Updating edges
		self.local_complement(qubit);
	}

	fn mzr_graph(&mut self, qubit: usize, measured: i32) {
		let neighbors: Vec<usize> = self.graph.edges[qubit].iter().cloned().collect();
		// Updating VOPs on neighbors and removing edges
		for n in neighbors {
			self.graph.remove_edge(n, qubit);
			if measured == 1 {
				self.right_apply_gate(n, ZGATE);
			}
		}

		// Updating remaining VOP
		if measured == 1 {
			self.right_apply_gate(qubit, XGATE);
		}
		self.right_apply_gate(qubit, HGATE);
	}

	pub fn debug_circuit(&self) -> String {
		let mut circ: String = String::from("@pragma total_num_qubits ") + &self.num_qubits.to_string() + "\n";
		circ += &(String::from("@pragma total_num_cbits 0\n"));
		let mut gate_decomps: Vec<Vec<String>> = Vec::new();
		gate_decomps.push(vec!["".to_string()]);
        gate_decomps.push(vec!["h".to_string(), "s".to_string(), "s".to_string(), "h".to_string()]);
        gate_decomps.push(vec!["s".to_string(), "s".to_string(), "h".to_string(), "s".to_string(), "s".to_string(), "h".to_string()]);
        gate_decomps.push(vec!["s".to_string(), "s".to_string()]);
        gate_decomps.push(vec!["s".to_string(), "h".to_string(), "s".to_string(), "h".to_string()]);
        gate_decomps.push(vec!["h".to_string(), "s".to_string(), "s".to_string(), "h".to_string(), "s".to_string(), "h".to_string(), "s".to_string(), "h".to_string()]);
        gate_decomps.push(vec!["s".to_string(), "s".to_string(), "h".to_string(), "s".to_string(), "s".to_string(), "h".to_string(), "s".to_string(), "h".to_string(), "s".to_string(), "h".to_string()]);
        gate_decomps.push(vec!["s".to_string(), "s".to_string(), "s".to_string(), "h".to_string(), "s".to_string(), "h".to_string()]);
        gate_decomps.push(vec!["s".to_string(), "h".to_string()]);
        gate_decomps.push(vec!["h".to_string(), "s".to_string(), "s".to_string(), "h".to_string(), "s".to_string(), "h".to_string()]);
        gate_decomps.push(vec!["s".to_string(), "s".to_string(), "h".to_string(), "s".to_string(), "s".to_string(), "h".to_string(), "s".to_string(), "h".to_string()]);
        gate_decomps.push(vec!["s".to_string(), "s".to_string(), "s".to_string(), "h".to_string()]);
        gate_decomps.push(vec!["h".to_string()]);
        gate_decomps.push(vec!["h".to_string(), "s".to_string(), "s".to_string()]);
        gate_decomps.push(vec!["s".to_string(), "s".to_string(), "h".to_string(), "s".to_string(), "s".to_string()]);
        gate_decomps.push(vec!["s".to_string(), "s".to_string(), "h".to_string()]);
        gate_decomps.push(vec!["s".to_string(), "h".to_string(), "s".to_string()]);
        gate_decomps.push(vec!["h".to_string(), "s".to_string(), "s".to_string(), "h".to_string(), "s".to_string(), "h".to_string(), "s".to_string()]);
        gate_decomps.push(vec!["s".to_string(), "s".to_string(), "h".to_string(), "s".to_string(), "s".to_string(), "h".to_string(), "s".to_string(), "h".to_string(), "s".to_string()]);
        gate_decomps.push(vec!["s".to_string(), "s".to_string(), "s".to_string(), "h".to_string(), "s".to_string()]);
        gate_decomps.push(vec!["s".to_string()]);
        gate_decomps.push(vec!["h".to_string(), "s".to_string(), "s".to_string(), "h".to_string(), "s".to_string()]);
        gate_decomps.push(vec!["s".to_string(), "s".to_string(), "h".to_string(), "s".to_string(), "s".to_string(), "h".to_string(), "s".to_string()]);
        gate_decomps.push(vec!["s".to_string(), "s".to_string(), "s".to_string()]);     

		// Apply initial Hadamards
		for i in 0..self.num_qubits {
			circ += &String::from(format!("h q{}\n", i));
		}

		// Apply edges
		for i in 0..self.num_qubits {
			for j in &self.graph.edges[i] {
				if i <= *j {
					circ += &String::from(format!("cz q{} q{}\n", i, j));
				}
			}
		}

		// Apply VOPs
		for i in 0..self.num_qubits {
			for gate in &gate_decomps[self.graph.vals[i]] {
				if gate != "" {
					circ += &String::from(format!("{} q{}\n", gate, i));
				}
			}
		}

		return circ;
	}
}

impl QuantumState for QuantumGraphState {
    fn new(num_qubits: usize) -> QuantumGraphState {
		let mut graph: Graph<usize> = Graph::new();

		for _i in 0..num_qubits {
			graph.add_vertex(HGATE);
		}

		return QuantumGraphState { num_qubits: num_qubits, graph: graph , rng: rand::thread_rng() };
	}

	fn print(&self) -> String { 
		return format!("Graph:\n{}\nVops:{:?}\n", self.graph.print(), self.graph.vals);
	}

	fn system_size(&self) -> usize {
		return self.num_qubits;
	}

    fn x_gate(&mut self, qubit: usize) {
		assert!(qubit < self.num_qubits);
		self.apply_gate(qubit, XGATE);
	}
    fn y_gate(&mut self, qubit: usize) {
		assert!(qubit < self.num_qubits);
		self.apply_gate(qubit, YGATE);
	}
    fn z_gate(&mut self, qubit: usize) {
		assert!(qubit < self.num_qubits);
		self.apply_gate(qubit, ZGATE);
	}

    fn h_gate(&mut self, qubit: usize) {
		assert!(qubit < self.num_qubits);
		self.apply_gate(qubit, HGATE);
	}

    fn s_gate(&mut self, qubit: usize) {
		assert!(qubit < self.num_qubits);
		self.apply_gate(qubit, SGATE);
	}

    fn cz_gate(&mut self, qubit1: usize, qubit2: usize) {
		assert!(qubit1 < self.num_qubits && qubit2 < self.num_qubits && qubit1 != qubit2);

		if !self.isolated(qubit1, qubit2) {
			self.remove_vop(qubit1, qubit2);
		}
		if !self.isolated(qubit2, qubit1) {
			self.remove_vop(qubit2, qubit1);
		}
		if !self.isolated(qubit1, qubit2) {
			self.remove_vop(qubit1, qubit2);
		}


		let lookup: (bool, usize, usize) = CZ_LOOKUP[self.graph.vals[qubit1]][self.graph.vals[qubit2]][self.graph.contains_edge(qubit1, qubit2) as usize];
		self.graph.vals[qubit1] = lookup.1;
		self.graph.vals[qubit2] = lookup.2;
		if lookup.0 != self.graph.contains_edge(qubit1, qubit2) {
			self.graph.toggle_edge(qubit1, qubit2);
		}
	}

    fn mzr_qubit(&mut self, qubit: usize) -> i32 {
		let basis: usize = CONJUGATION_TABLE[self.graph.vals[qubit]];
		let positive: i32 = (basis > 3) as i32;
		let measured: i32 = match basis {
			1 | 4 => {
				if self.graph.degree(qubit) == 0 {
					return positive // If measuring in X-basis on isolated vertex, return immediately
				} else {
					(self.rng.gen::<u8>() % 2) as i32
				}
			} _ => (self.rng.gen::<u8>() % 2) as i32,
		};

		match basis {
			1 | 4 => self.mxr_graph(qubit, measured),
			2 | 5 => self.myr_graph(qubit, measured),
			3 | 6 => self.mzr_graph(qubit, measured),
			_ => panic!()
		};

		return measured ^ positive;
	}
}

impl Entropy for QuantumGraphState {
	fn renyi_entropy(&self, qubits: &Vec<usize>) -> f32 {
		let mut bipartite_graph = self.graph.partition(&qubits);
		let mut entropy: f32 = 2.*(bipartite_graph.num_vertices as f32)
							   - bipartite_graph.vals.iter().filter(|&x| *x ).count() as f32;

		

		while bipartite_graph.num_vertices > 0 {
			let del_node = bipartite_graph.num_vertices - 1;
			let del_node_degree = bipartite_graph.degree(del_node);
			match del_node_degree {
				0 => {
					if !bipartite_graph.vals[del_node] {
						entropy -= 2.;
					} else {
						entropy -= 1.;
					}
					bipartite_graph.remove_vertex(del_node);
				},
				1 => {
					let neighbor = bipartite_graph.edges[del_node][0];
					if bipartite_graph.vals[del_node] {
						let mut del_nodes: Vec<usize> = Vec::new();
						for j in 0..bipartite_graph.degree(neighbor) {
							let node = bipartite_graph.edges[neighbor][j];
							if bipartite_graph.degree(node) == 1 {
								del_nodes.push(node);
							}
						}

						del_nodes.push(neighbor);
						del_nodes.sort();
						del_nodes.reverse();

						for j in 0..del_nodes.len() {
							bipartite_graph.remove_vertex(del_nodes[j]);
						}
						entropy -= del_nodes.len() as f32;
					} else {
						bipartite_graph.remove_vertex(del_node);
						bipartite_graph.remove_vertex(neighbor);
						entropy -= 2.;
					}
				},
				_ => {
					// Need to identify if a pivot exists
					let mut found_pivot: bool = false;
					let mut pivot: usize = 0;
					let mut min_degree: usize = !0;
					for j in 0..del_node_degree {
						let neighbor = bipartite_graph.edges[del_node][j];
						let degree = bipartite_graph.degree(neighbor);

						if degree != 1 && degree < min_degree {
							found_pivot = true;
							min_degree = degree;
							pivot = neighbor;
						}
					}

					// there is no valid pivot, so graph is a simple tree; clear it
					if !found_pivot {
						let mut sorted_neighbors: Vec<usize> = bipartite_graph.edges[del_node].iter().cloned().collect();
						sorted_neighbors.sort();
						sorted_neighbors.reverse();
						let delA: bool = bipartite_graph.vals[del_node];

						bipartite_graph.remove_vertex(del_node);
						for j in 0..del_node_degree {
							bipartite_graph.remove_vertex(sorted_neighbors[j]);
						}

						if !delA {
							entropy -= 1. + del_node_degree as f32;
						} else {
							entropy -= 2.*del_node_degree as f32;
						}

					} else {
						let mut toggles: Vec<(usize, usize)> = Vec::new();
						for j in 0..del_node_degree {
							let neighbor = bipartite_graph.edges[del_node][j];
							if neighbor != pivot {
								for k in 0..min_degree {
									let pivot_neighbor = bipartite_graph.edges[pivot][k];
									toggles.push((neighbor, pivot_neighbor));
								}
							}
						}

						for toggle in &toggles {
							bipartite_graph.toggle_edge(toggle.0, toggle.1);
						}

						// Pivot completed; ready to be deleted on next iteration
					}

				}
			};

		}


		return entropy as f32;
	}
}