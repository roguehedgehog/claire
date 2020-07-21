variable aws_profile {
  default = "default"
}

variable aws_region {
  type = string
}

variable "prefix" {
  type = string
}


variable "claire_az" {
  type = string
}


variable "claire_cidr" {
  default = "10.99.0.0/16"
}

variable "claire_public_cidr" {
  default = "10.99.0.0/16"
}
