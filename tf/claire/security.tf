resource "aws_security_group" "locked_down" {
  name        = "CLAIRE Locked Down"
  description = "Instances assigned to this group cannot communicate on the network"

  egress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_security_group" "egress_only" {
  name        = "CLAIRE Investigator"
  description = "To allow investigator services to talk to S3 and the internet"

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

