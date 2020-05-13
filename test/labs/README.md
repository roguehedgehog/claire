
# Welcome to CLAIRE Labs!

## Getting Started

Generate a keypair and add it to your AWS account

```
 ssh-keygen -t rsa -C "claire-lab-key" -f ~/.ssh/claire-lab-key
 aws ec2 import-key-pair --key-name "claire lab key" --public-key-material fileb://~/.ssh/claire-lab-key.pub
 ```

 Deploy the lab scope

 ```
 cdk deploy
 ```