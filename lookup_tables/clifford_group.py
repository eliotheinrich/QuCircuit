import numpy as np
from scipy.linalg import sqrtm
from enum import Enum
import pickle as pkl

I = np.diag([1, 1])
S = np.diag([1, 1j])
H = np.array([[1,1],[1,-1]])/np.sqrt(2)
W = H @ S
V = W @ W

X = np.array([[0,1],[1,0]])
Y = np.array([[0,-1j],[1j,0]])
Z = np.diag([1,-1])

sqrtX = sqrtm(X)
sqrtXd = np.conjugate(sqrtX.T)

sqrtY = sqrtm(Y)
sqrtYd = np.conjugate(sqrtY.T)

sqrtZ = S
sqrtZd = np.conjugate(S.T)

ZGATES = set([0, 3, 20, 23])

A = [I, V, W, H, H @ V, H @ W]
B = [I, X, Y, Z]

C1 = []
def fix_gauge(A):
	if np.isclose(np.abs(A[0,0]), 0):
		return A / (A[0,1]/np.abs(A[0,1]))
	else:
		return A / (A[0,0]/np.abs(A[0,0]))
for a in A:
	for b in B:
		C1.append(fix_gauge(a @ b))

NUM_CLIFFORDS = len(C1)
operators = {'I': I, 'S': S, 'H': H, 'Sd': np.conjugate(S.T), 'X': X, 'Y': Y, 'Z': Z,
			 'sqrtX': sqrtX, 'sqrtXd': sqrtXd, 'sqrtY': sqrtY, 'sqrtYd': sqrtYd,
			 'sqrtZ': sqrtZ, 'sqrtZd': sqrtZd, 'V': V, 'W': W, 'X H': X @ H ,
			 'X2': X, 'Z2': Z}


eps = 1e-6
def arr_prop(A1, A2):
	A1_f = A1.flatten()
	A1_nonzero = np.where(np.abs(A1_f) > eps)
	A2_f = A2.flatten()
	A2_nonzero = np.where(np.abs(A2_f) > eps)

	if np.array_equal(A1_nonzero, A2_nonzero):
		C = A1_f[A1_nonzero] / A2_f[A2_nonzero]
		return np.all(np.isclose(C, C[0]))
	else:
		return False