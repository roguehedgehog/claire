resource "aws_iam_instance_profile" "lab_profile" {
  name = "lab_profile"
  role = aws_iam_role.lab_role.name
}

resource "aws_iam_role" "lab_role" {
  name               = "lab_role"
  assume_role_policy = data.aws_iam_policy_document.lab_role.json
}

data "aws_iam_policy_document" "lab_role" {
  statement {
    actions = [
      "sts:AssumeRole",
    ]

    principals {
      type = "Service"
      identifiers = [
        "ec2.amazonaws.com",
      ]
    }
  }
}

data "aws_iam_policy" "AmazonEc2RoleForSSM" {
  arn = "arn:aws:iam::aws:policy/service-role/AmazonEC2RoleforSSM"
}

resource "aws_iam_role_policy_attachment" "evidence_extractor_ssm_access" {
  role       = aws_iam_role.lab_role.id
  policy_arn = data.aws_iam_policy.AmazonEc2RoleForSSM.arn
}
