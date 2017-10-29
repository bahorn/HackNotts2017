import os
import requests

class NessieApi:
    def __init__(self, key, baseurl='http://api.reimaginebanking.com'):
        self.key = key
        self.baseurl = baseurl
    def customers(self):
        r = requests.get('{}/customers?key={}'.format(self.baseurl,self.key))
        return r.json()

if __name__ == "__main__":
    try:
        key = os.environ['API_KEY']
        l = NessieApi(key)
        for i in l.customers():
            print i['first_name'],i['last_name'],'-',i['_id']
    except KeyError:
        print('provide a ENV variable called API_KEY, with an API_KEY')
