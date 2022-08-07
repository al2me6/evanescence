import json
import sys

from matplotlib import pyplot as plt

with open(sys.argv[1], "r", encoding="utf8") as file:
    data = json.load(file)
    rhos = data["rhos"]
    cdf = data["cdf"]
    data_len = len(rhos[0])
    min_expected = cdf[0]
    max_expected = cdf[-1]
    plt.plot(rhos[0], cdf)
    rho_cdf = [
        (i + 1) / data_len * (max_expected - min_expected) + min_expected for i in range(data_len)
    ]
    for rho in rhos:
        plt.plot(rho, rho_cdf)
    plt.title(f"{data['name']}, worst K-S = {data['ks']}, worst p = {data['p']}")
    plt.show()
