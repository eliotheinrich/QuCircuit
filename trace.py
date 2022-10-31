import numpy as np
from scipy.linalg import sqrtm

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

A = [I, V, W, H, H @ V, H @ W]
B = [I, X, Y, Z]

C1 = []
for a in A:
	for b in B:
		C1.append(a @ b)

NUM_CLIFFORDS = len(C1)

class GraphState:
	def __init__(self, num_qubits = 0):
		self.num_qubits = num_qubits
		self.edges = [set() for i in range(num_qubits)]
		self.vops = [12 for i in range(num_qubits)]
		self.visited = set([])

	def add_edge(self, a, b):
		if b not in self.edges[a]:
			self.edges[a].add(b)
			self.edges[b].add(a)
	
	def remove_edge(self, a, b):
		if b in self.edges[a]:
			self.edges[a].remove(b)
			self.edges[b].remove(a)
	
	def adjacency_matrix(self):
		L = np.zeros((self.num_qubits, self.num_qubits))
		for i in range(self.num_qubits):
			for j in range(self.num_qubits):
				if i in self.edges[j]:
					L[i,j] = 1
		return L
	
	
	@staticmethod
	def debug_state():
		G = GraphState(10)

		G.add_edge(0, 1)

		G.add_edge(1, 2)
		G.add_edge(1, 3)

		G.add_edge(2, 4)
		G.add_edge(2, 5)
		G.add_edge(2, 9)

		G.add_edge(3, 9)

		G.add_edge(4, 5)
		G.add_edge(4, 9)

		G.add_edge(5, 7)
		G.add_edge(5, 9)

		G.add_edge(6, 7)

		G.add_edge(7, 9)

		return G


def gate(M, qubit, total_qubits):
	return np.kron(np.eye(2**(qubit)), np.kron(M, np.eye(2**(total_qubits - qubit - 1))))

def make_vector_state(graph_state):
	N = graph_state.num_qubits
	p = np.zeros(2**N, complex)
	p[0] = 1.

	for i in range(N):
		p = gate(H, i, N) @ p
	
	for i in range(N):
		for j in graph_state.edges[i]:
			if i < j:
				for n in range(2**N):
					if (n >> i) & 1 and (n >> j) & 1:
						p[n] *= -1

	for i in range(N):
		p = gate(C1[graph_state.vops[i]], i, N) @ p
	
	return p


def partial_trace(rho, size_A):
	rhoA = np.zeros((2**size_A,2**size_A),dtype=complex)

	d = len(rho)//2**size_A

	p = np.zeros(d)
	for i in range(d):
		p[i-1] = 0
		p[i] = 1

		P = np.kron(p, np.eye(2**size_A))

		rhoA += P @ rho @ P.T


	print(rhoA)
	return rhoA


def entropy(G, num_qbits):
	p = make_vector_state(G)
	rho = np.outer(p, np.conjugate(p))
	rhoA = partial_trace(rho, num_qbits)
	return -np.rint(np.real(np.log2(np.trace(rhoA @ rhoA))))

def CZ(q1, q2, num_qbits):
	return np.array([-1 if (i >> q1) & 1 and (i >> q2) & 1 else 1 for i in range(2**num_qbits)])

def nth_bit(i, n):
	return (i >> n) & 1

def set_bit(v, index, x):
  mask = 1 << index   
  v &= ~mask          
  if x:
    v |= mask         
  return v


graph = 2
if graph == 0:
	# Loop
	G = GraphState(6)
	G.add_edge(0, 3)
	G.add_edge(0, 5)

	G.add_edge(1, 3)
	G.add_edge(1, 4)

	G.add_edge(2, 4)
	G.add_edge(2, 5)

	A = 3
elif graph == 1:
	# Funky
	G = GraphState(8)
	G.add_edge(0, 3)

	G.add_edge(1, 4)
	G.add_edge(1, 5)

	G.add_edge(2, 5)
	G.add_edge(2, 6)
	G.add_edge(2, 7)

	A = 3
elif graph == 2:
	# Trivial
	G = GraphState(3)
	G.add_edge(0, 2)

	A = 2

N = len(G.edges)

print(f'safe entropy: {entropy(G, A)}')

s = 0
for x in range(2**N):
	for y in range(2**N):
		ex = 0.
		for a,ngbh in enumerate(G.edges[0:A]):
			for b in ngbh:
				ex += (nth_bit(x, a) + nth_bit(y, a))*(nth_bit(x, b) + nth_bit(y, b))
		
		s += (-1)**ex
s = -np.real(np.log2(s/(2**(2*N))))
print(f'testing: {s}')

s = 0
L = G.adjacency_matrix()
for i in range(2**N):
	for j in range(2**N):
		iv = np.array([int(x) for x in list(f'{i:0{N}b}')])
		jv = np.array([int(x) for x in list(f'{j:0{N}b}')])
		s += (-1)**((iv + jv) @ L @ (iv + jv)/2)
s = -np.real(np.log2(s/(2**(2*N))))
print(f'safe-ish entropy: {s}')

def entropy3(G):
	B = N - A
	s = 0
	for x in range(2**B):
		for y in range(2**B):
			g = 1
			for a,edges in enumerate(G.edges[0:A]):
				ex = 0
				for b in edges:
					ex += nth_bit(x, b - A) + nth_bit(y, b - A)
				g *= (1 + (-1)**ex)**2
			s += g
	return -np.log2(s/2**(2*N))

print(entropy3(G))

