#!/usr/bin/env python
import boto3
import sys,os

# This tool uploads blobs to digital ocean spaces / s3

if __name__ == "__main__":
    if len(sys.argv) < 4:
        print("Usage: ./pushcontrol.py <bucket> <uuid> <data>")
        exit(-1)
    bucket_name = sys.argv[1]
    uuid = sys.argv[2]
    data = sys.argv[3]
    secret_key_id = os.environ['AWS_SECRET_KEY_ID']
    secret_key = os.environ['AWS_SECRET_KEY']
    url = "dat/{}.blob".format(uuid)
    session = boto3.session.Session()
    client = boto3.resource('s3',
                region_name='nyc3',
                endpoint_url='https://nyc3.digitaloceanspaces.com',
                aws_access_key_id=secret_key_id,
                aws_secret_access_key=secret_key)
    client.Object(bucket_name, url).put(ACL='public-read', Body=data)
