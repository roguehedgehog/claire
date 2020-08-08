resource "aws_security_group" "locked_down" {
  name        = "claire_locked_down"
  vpc_id      = var.vpc_id
  description = "Instances should only be able to communicate with SSM"

  egress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = var.allowed_cidr_when_lockeddown
  }
}

resource "aws_security_group" "egress_only" {
  name        = "claire_investigator"
  vpc_id      = var.vpc_id
  description = "To allow investigator services to talk to the internet"

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_security_group" "egress_and_ssh" {
  name        = "claire_manual_investigator"
  vpc_id      = var.vpc_id
  description = "To allow investigator services to talk to the internet"

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    protocol    = "tcp"
    from_port   = 22
    to_port     = 22
    cidr_blocks = ["0.0.0.0/0"]
  }
}

