# claire

claire was developed as part of my MSc in Information Security from Royal Holloway; and is not my girlfriend. 

## Automating Incident Response

There are 3 main parts to this project the first is [serverless application](tf/claire) build with 
AWS Step Functions and Lambdas written in Python to respond to incidents on EC2 instances by 

  - [Tagging the instance and capturing instance metadata](tf/claire/lambda/create_investigation.py) 
  - [Taking snapshots of attached volumes](tf/claire/lambda/snapshot_volumes.py)
  - Provisions a volume to [capture memory](tf/claire/lambda/extract_memory.py) with [avml](https://github.com/microsoft/avml)
  - Provisioning [another instance](tf/claire/evidence_extractor) to investigate the incident
  - Uses the snapshots to create duplicate volumes to 
        [extract artifacts](tf/claire/evidence_extractor/bin/volume_root_capture.sh) and 
        [generate a timeline](tf/claire/evidence_extractor/bin/volume_timeline.sh) with [The Sleuth Kit](https://github.com/sleuthkit/sleuthkit)
  - Uses [Volatility](https://github.com/volatilityfoundation/volatility) to perform [memory analysis](tf/claire/evidence_extractor/bin/memory_analysis.sh)
  
  All the evidence is saved an S3 bucket.
  
 There is a [CLI written in Rust](claire), to trigger the incident response process, manage the investigations, and can also invalidate instance credentials.
 
 ```
 SUBCOMMANDS:
    clear           Removes the CLAIRE tags from investigated resources, clear the investigation but leave the
                    collected evidence
    download        Download investigation evidence to a local directory
    help            Prints this message or the help of the given subcommand(s)
    investigate     Starts an investigation into the given instance
    isolate         Remove existing security groups and apply restrictive security group
    list            List the investigations
    manual          Spin up an instance and attach snapshots of a suspicious instance so an investigation can be
                    continued manually.
    purge           Purge the investigation, removes evidence from S3, untags and deletes snapshots
    revoke          Revoke instance permissions and invalidate any tokens that may have been stolen.
    status          View the status of an investigation
    token-expire    Find the role assosciated with an instance profile and expire their tokens.
```

And finally there are the labs I used to [test claire](bin/lab-exploits), this is an instance with Drupal 8.50 installed on it vulnerable to 
CVE-2018-7600 on port 80 and a open forward proxy on port 81 used to access the Instance MetaData Service. 
AWS GuardDuty can [optionally be enabled](tf/claire/variables.tf#L31) which will detect these [exploits](bin/exploits) and automatically run the incident response process. 

## Getting Started

You will need

- awscli
- Rust
- Terraform
- Packer
- jq

`make install` will run packer to create the investigating instance AMI and then run terraform in tf/claire to setup the infrastructure for claire.
claire comes with [Volaility profiles for the lab machine, Ubuntu 18.04](tf/claire/evidence_extractor/profiles) 
if additional profiles are required these should be created in the profiles directory before `make install`is run, or run packer again to update the AMI.  

`make install-labs`will run packer to create the vulnerabele instnace AMI and then run the terraform in tf/labs to create the labs. 

Should go without saying, be careful running the labs, there is a security group created that will [only allow access from your IP](tf/labs/main.tf#L25), 
but still the instance is vulnerable to drive-by comprimise.

Take care, have fun :)

  

