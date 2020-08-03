variable "aws_profile" {
  description = "The AWS Profile to use to deploy CLAIRE"
  default     = "default"
}

variable "aws_region" {
  description = "The AWS region to use to the deploy CLAIRE"
  type        = string
}

variable "vpc_id" {
  description = "The VPC to be used to create security groups"
  type        = string
}

variable "allowed_cidr_when_lockeddown" {
  description = "If SSM VPC endpoints are enabled this can be the VPC CIDR otherwise 0.0.0.0/0"
  type        = list(string)
}

variable "prefix" {
  description = "The prefix for s3 buckets (for uniqueness)"
  type        = string
}

variable "evidence_extractor_ami_id" {
  description = "The Evidence Extractor AMI created by packer"
  type        = string
}
