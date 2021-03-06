variable "aws_region" {
  type = string
}

variable "lab_availability_zone" {
  type = string
}

variable "lab_ami_id" {
  type = string
}

variable "prefix" {
  type = string
}

variable "lab_cidr" {
  default = "10.99.0.0/20"
}

variable "create_vpc_endpoints" {
  default = 0
}

variable "lab_count" {
  default = 1
}

variable "lab_pub_key" {
  default = "dist/lab_key.pub"
}

variable "lab_pri_key" {
  default = "dist/lab_key"
}

variable "key_name" {
  default = "claire-lab-key"
}

variable "instance_type" {
  default = "t2.micro"
}
