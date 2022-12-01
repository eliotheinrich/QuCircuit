import numpy as np
import matplotlib.pyplot as plt
import json
from scipy.optimize import curve_fit

def combine_dist(s1, s2):
	(mean1, std1, N1) = s1
	(mean2, std2, N2) = s2

	N3 = N1 + N2
	mean3 = (N1*mean1 + N2*mean2)/N3
	std3 = np.sqrt((N1*(std1**2 + (mean1 - mean3)**2) + N2*(std2**2 + (mean2 - mean3)**2))/N3)

	return (mean3, std3, N3)

def combine_many_dists(dists):
	dists = list(dists)
	dist = dists[0]
	for d in dists[1:]:
		dist = combine_dist(d, dist)
	return dist

class DataSlide:
	def __init__(self, keys, vals):
		self.data = dict(zip(keys, vals))
		
	def get(self, key):
		if self.data[key].ndim == 1:
			return self.data[key][0]
		else:
			return self.data[key][:,0]
	
	def get_err(self, key):
		if self.data[key].ndim == 1:
			raise KeyError("Data not found")
		else:
			return self.data[key][:,1]

	def get_nruns(self, key):
		if self.data[key].ndim == 1:
			raise KeyError("Data not found")
		else:
			return self.data[key][:,2]

class DataFrame:
	def __init__(self):
		self.slides = []
	
	def __add__(self, other):
		new = DataFrame()
		for slide in self.slides:
			new.add_dataslide(slide)
		for slide in other.slides:
			new.add_dataslide(slide)
		return new

	def add_dataslide(self, slide):
		self.slides.append(slide)

	def get_property_with_id(self, key, id):
		return self.slides[id][key]

	def get(self, key):
		val = []
		for slide in self.slides:
			val.append(slide.get(key))
		return np.array(val)
	
	def get_err(self, key):
		val = []
		for slide in self.slides:
			val.append(slide.get_err(key))
		return np.array(val)

	def get_nruns(self, key):
		val = []
		for slide in self.slides:
			val.append(slide.get_nruns(key))
		return np.array(val)
	
	def query_key(self, key, val):
		new_df = DataFrame()
		for slide in self.slides:
			if np.isclose(slide[key], val):
				new_df.add_dataslide(slide)
		return new_df

def parse_datafield(s):
	if list(s.keys())[0] == 'Data':
		sample = s[list(s.keys())[0]]
		return np.array(sample)
	else:
		return np.array([s[list(s.keys())[0]]])

def load_data(filename):
	data = DataFrame()
	with open(filename, 'r') as f:
		json_contents = json.load(f)
		for slide in json_contents['slides']:
			keys = list(slide['data'].keys())
			vals = [parse_datafield(slide['data'][key]) for key in keys]
				
			data.add_dataslide(DataSlide(keys, vals))
	
	return data

def plot_run(data: DataFrame, run_id: int, average_interval: int = 1, ax = None):
	if ax is None:
		ax = plt.gca()

	entropy = data.get_property_with_id("entropy", run_id)
	if average_interval != 1:
		times = []
		entropy_avg = []
		for i in range(len(entropy)//average_interval):
			times.append(i*average_interval)
			entropy_avg.append(np.mean(entropy[i*average_interval:(i+1)*average_interval]))
	else:
		entropy_avg = entropy

	ax.plot(entropy_avg)
	ax.set_xlabel(r'$t$', fontsize=16)
	ax.set_ylabel(r'$S_A^2$', fontsize=16)

def plot_all_data(data: DataFrame, ax = None):
	if ax is None:
		ax = plt.gca()

	unique_p = sorted(list(set(data.get('mzr_prob'))))
	unique_LA = sorted(list(set(data.get('partition_size'))))
	entropy = np.zeros((len(unique_p), len(unique_LA)))
	entropy_err = np.zeros((len(unique_p), len(unique_LA)))
	for slide in data.slides:
		i = unique_p.index(slide.get('mzr_prob'))
		j = unique_LA.index(slide.get('partition_size'))

		entropy[i][j] = slide.get('entropy')
		entropy_err[i][j] = slide.get_err('entropy')

	colors = ['C0', 'orange', 'yellow', 'purple', 'green', 'black', 'magenta', 'cyan']
	for n,p in enumerate(unique_p):
		if p != 0:
			ax.errorbar(unique_LA, entropy[n], yerr=entropy_err[n], linewidth=1.5, marker='*', color=colors[n-1 if 0 in unique_p else n], label=f'p = {p}')

	ax.legend(fontsize=16)
	ax.set_xlabel(r'$L_A$', fontsize=16)
	ax.set_ylabel(r'$\overline{S_A^{(2)}}$', fontsize=16)

def linear(x, a, b):
	return x*a + b

def average_data(df: DataFrame, measurement_freq: int = 1):
	entropy_samples = np.array([slide.get('entropy') for slide in df.slides]).T
	entropy_err_samples = np.array([slide.get_err('entropy') for slide in df.slides]).T
	nrun_samples = np.array([slide.get_nruns('entropy') for slide in df.slides]).T

	num_times = len(entropy_samples)

	S = np.zeros(num_times)
	dS = np.zeros(num_times)
	N = np.zeros(num_times)

	for i in range(num_times):
		(S[i], dS[i], N[i]) = combine_many_dists(zip(entropy_samples[i], entropy_err_samples[i], nrun_samples[i]))
	
	t = np.array([i*measurement_freq for i in range(1, 1 + num_times)])

	return S, dS, N, t

def fig1(filename, ax=None):
	if ax is None:
		ax = plt.gca()
	
	data = load_data(filename)
	plot_all_data(data)

def fig2(filenames, ax=None, linear_fit=False, timeseries_filenames=None):
	if ax is None:
		ax = plt.gca()
	data = []
	for filename in filenames:
		data.append(load_data(filename))
	
	xs = {}
	Ss = {}
	for df in data:
		xs[df.slides[0].get('system_size')] = []
		Ss[df.slides[0].get('system_size')] = []
		for slide in df.slides:
			xs[slide.get('system_size')].append(slide.get('partition_size'))
			Ss[slide.get('system_size')].append(np.mean(slide.get('entropy')))
		xs[df.slides[0].get('system_size')] = np.array(xs[df.slides[0].get('system_size')])
		Ss[df.slides[0].get('system_size')] = np.array(Ss[df.slides[0].get('system_size')])
	
	for L, LA in xs.items():
		xs[L] = np.log(np.sin(np.pi*LA/L)*L/np.pi)

	for L, logx in xs.items():
		inds = np.argsort(xs[L])
		xs[L] = xs[L][inds]
		Ss[L] = Ss[L][inds]

		if linear_fit:
			fit_ind = 10
			p = curve_fit(linear, xs[L][fit_ind:], Ss[L][fit_ind:])[0]
			plt.plot(xs[L][fit_ind:], p[0]*xs[L][fit_ind:] + p[1], linestyle='--', label=f'L = {L} fit')
			print(f'{L}: {p}')
		plt.plot(xs[L], Ss[L], label=f'L = {L}')


	time = timeseries_filenames is not None
	xlabel = r'$\log(x), \log(t)$' if time else r'$\log(x)$'
	ax.set_xlabel(xlabel, fontsize=16)

	ax.set_ylabel(r'$\overline{S_A^{(2)}}$', fontsize=16)
	ax.legend(fontsize=16)
	if time:
		timedata = [load_data(f) for f in timeseries_filenames]

		(S1, dS1, N1, t1) = average_data(timedata[0], 1)
		(S2, dS2, N2, t2) = average_data(timedata[1], 10)

		S, dS, N, t = np.concatenate((S1, S2)), np.concatenate((dS1, dS2)), np.concatenate((N1, N2)), np.concatenate((t1, t2))

		inds = np.argsort(t)
		S, dS, N, t = S[inds], dS[inds], N[inds], t[inds]
		
		logt = np.log(t)

		ax.plot(logt, S)
		if linear_fit:
			fit_ind = 15
			p = curve_fit(linear, logt[fit_ind:], S[fit_ind:])[0]
			ax.plot(logt[fit_ind:], p[0]*logt[fit_ind:] + p[1], linestyle='--')
			print(f'800 logt: {list(p)}')

def fig3(filename, ax=None):
	if ax is None:
		ax = plt.gca()
	data = load_data(filename)
	S, dS, N, t = average_data(data)
	logt = np.log(t)
	var = dS**2

	L = data.get('system_size')[0]

	norm = S[10] / (dS**2)[10]
	ax.plot(t, var, marker='*', label=r'Var($\overline{S_A^{(2)}}$)')
	ax.plot(t, S, marker='*', label=r'$\overline{S_A^{(2)}}$')
	ax.text(0.75, 0.75, r'$p = 0.138$', transform=ax.transAxes, fontsize=16)
	ax.text(0.75, 0.65, r'$L = 800$', transform=ax.transAxes, fontsize=16)
	ax.text(0.75, 0.55, r'$L_A = 400$', transform=ax.transAxes, fontsize=16)
	ax.set_xlabel(r'$t$', fontsize=16)
	ax.set_ylabel(r'$\overline{S_A^{(2)}}$,   Var($\overline{S_A^{(2)}}$)', fontsize=16)
	ax.legend(fontsize=16)

	fit_ind = 15
	p = curve_fit(linear, logt[fit_ind:], var[fit_ind:])[0]
	print(f'800 logt: {list(p)}')

	fit_ind = 15
	p = curve_fit(linear, logt[fit_ind:], S[fit_ind:])[0]
	print(f'800 logt: {list(p)}')


#fig1("data/base.json", ax=plt.gca())
#plt.show()


filenames = ['data/fig2_1.json', 'data/fig2_2.json', 'data/fig2_3.json']
filenames = []
timeseries_filenames = ['data/timeseries.json', 'data/timeseries_small.json']
#fig2(filenames, ax=plt.gca(), linear_fit=True, timeseries_filenames=['data/timeseries_small.json', 'data/timeseries.json'])
#plt.show()

fig3('data/timeseries.json', plt.gca())
plt.show()

