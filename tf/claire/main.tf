terraform {
  required_version = ">=0.12"
}

provider "aws" {
  version = "~>2.0"
  profile = var.aws_profile
  region  = var.aws_region
}


