import json
import requests

class ControlProxyDelivery:
    cookie = ""
    def __init__(self, username, password, register=False, site="http://localhost:8000/"):
        self.site = site
        self.s = requests.Session()
        if register == False:
            r = self.s.post("{}/auth".format(self.site), json={"username":username,"password":password})
        else:
            r = self.s.post("{}/register".format(self.site), json={"username":username,"password":password})
        resp = r.json()
    # get our logged in status
    def info(self):
        r = self.s.get("{}/info".format(self.site))
        return r.json()
    # get a list of our queue
    def list(self):
        r = self.s.get("{}/list_blobs".format(self.site))
        return r.json()
    # add to our queue
    def add(self, blob):
        r = self.s.post("{}/add_blob".format(self.site), json={"blob":blob})
        return r.json()
    # drop from our queue
    def delete(self, uuid):
        r = self.s.post("{}/del_blob".format(self.site), json={"uuid":uuid})
        return r.json()
    # deploy live
    def push(self, uuid, name):
        r = self.s.post("{}/push_blob".format(self.site), json={"uuid":uuid,"name":name})
        return r.json()
    def logout(self):
        r = self.s.post("{}/logout".format(self.site), json={"valid":true})
        return r.json()


if __name__ == "__main__":
    control = ControlProxyDelivery("admin","d0nth4ckm3")
    print control.info()
    print control.list()
    message = open('./secret').read()
    print control.add(message)
    for i in control.list():
        if i['value'] == message:
            control.push(i['uuid'],"known_value")
