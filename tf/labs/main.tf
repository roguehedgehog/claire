terraform {
  required_version = ">=0.12"
}

provider "aws" {
  version = "~>2.0"
  region  = var.aws_region
}

resource "aws_instance" "lab" {
  count                       = var.lab_count
  ami                         = var.lab_ami_id
  instance_type               = var.instance_type
  key_name                    = var.key_name
  subnet_id                   = aws_subnet.lab_subnet.id
  vpc_security_group_ids      = [aws_security_group.lab_security_group.id]
  iam_instance_profile        = aws_iam_instance_profile.lab_profile.name
  associate_public_ip_address = true
  tags = {
    Name = "CLAIRE Lab Vulnerable Instance"
    Lab  = "CLAIRE"
  }
}

resource "aws_security_group" "lab_security_group" {
  name   = "claire_lab_security_group"
  vpc_id = aws_vpc.lab_vpc.id

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



