terraform {
  required_version = ">=0.12"
}

provider "aws" {
  version = "~>2.0"
  region  = var.aws_region
}

resource "aws_instance" "lab" {
  count                  = var.lab_count
  ami                    = var.lab_ami_id
  instance_type          = var.instance_type
  key_name               = var.key_name
  vpc_security_group_ids = [aws_security_group.vuln_group.id]
  iam_instance_profile   = aws_iam_instance_profile.lab_profile.name
  tags = {
    Name = "CLAIRE Lab Vulnerable Instance"
    Lab  = "CLAIRE"
  }
}

resource "aws_security_group" "vuln_group" {
  name = "vuln_security_group"

  ingress {
    protocol    = "tcp"
    from_port   = 22
    to_port     = 22
    cidr_blocks = ["${chomp(data.http.my_external_ip.body)}/32"]
  }

  ingress {
    protocol    = "tcp"
    from_port   = 80
    to_port     = 81
    cidr_blocks = ["${chomp(data.http.my_external_ip.body)}/32"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_key_pair" "lab_key" {
  key_name   = var.key_name
  public_key = file(var.lab_pub_key)
}



